use anyhow::Result;
use azure_identity::DefaultAzureCredential;
use azure_security_keyvault::prelude::KeyVaultGetSecretsResponse;
use azure_security_keyvault::KeyvaultClient;
use clap::Parser;
use futures::stream::StreamExt;
use paris::{info, log};
use paris::{error, Logger};
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Semaphore;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Opts {
    /// Sets the name of the Azure Key Vault
    #[clap(short, long, value_name = "VAULT_NAME")]
    vault_name: String,

    /// Sets the name of the output file
    #[clap(short, long, value_name = "FILE", default_value = ".env")]
    output: String,

    /// Filters the secrets to be retrieved by name
    #[clap(short, long, value_name = "FILTER")]
    filter: Option<String>,
}

async fn check_vault_dns(vault_name: &str) -> Result<()> {
    let vault_host = format!("{}.vault.azure.net", vault_name);

    let lookup_result = {
        tokio::net::lookup_host((vault_host.as_str(), 443)).await
    };

    match lookup_result {
        Ok(_) => Ok(()),
        Err(err) => {
            error!("DNS lookup failed for Key Vault: {}", vault_name);
            info!("Please check that the Key Vault exists or that you have no connectivity issues.");
            Err(err.into())
        }
    }
}


async fn fetch_secrets_from_key_vault(
    client: &KeyvaultClient,
    filter: Option<&str>,
) -> Result<Vec<(String, String)>> {
    let mut secret_values = Vec::new();
    let mut secret_pages = client.secret_client().list_secrets().into_stream();

    while let Some(page) = secret_pages.next().await {
        let page = match page {
            Ok(p) => p,
            Err(err) => {
                log!("\n");
                error!("Failed to fetch secrets page: {}", err);
                return Err(err.into());
            }
        };
        secret_values
            .extend(fetch_secrets_from_page(&client.secret_client(), &page, filter).await?);
    }

    Ok(secret_values)
}

async fn fetch_secrets_from_page(
    client: &azure_security_keyvault::SecretClient,
    page: &KeyVaultGetSecretsResponse,
    filter: Option<&str>,
) -> Result<Vec<(String, String)>> {
    let (tx, mut rx) = mpsc::channel(32);
    let semaphore = Arc::new(Semaphore::new(10));
    let mut handles = Vec::new();

    for secret in &page.value {
        if let Some(filter) = filter {
            if !secret.id.contains(filter) {
                continue;
            }
        }

        let permit = semaphore.clone().acquire_owned().await.unwrap();
        let tx = tx.clone();
        let secret_id = secret.id.clone();
        let client_clone = client.clone();

        handles.push(tokio::spawn(async move {
            let _permit = permit;
            fetch_and_send_secret(client_clone, secret_id, tx).await
        }));
    }

    drop(tx);

    let mut secrets = Vec::new();
    for handle in handles {
        if let Ok(result) = handle.await {
            secrets.push(result);
        } else {
            error!("Error occurred while fetching a secret.");
        }
    }

    while let Some(result) = rx.recv().await {
        secrets.push(result);
    }

    Ok(secrets)
}

async fn fetch_and_send_secret(
    client: azure_security_keyvault::SecretClient,
    secret_id: String,
    tx: mpsc::Sender<(String, String)>,
) -> (String, String) {
    let secret_name = secret_id.split('/').last().unwrap_or_default();
    match client.get(secret_name).await {
        Ok(bundle) => {
            let _ = tx.send((secret_id.clone(), bundle.value.clone())).await;
            (secret_id, bundle.value)
        }
        Err(err) => {
            error!("Error fetching secret: {}", err);
            (secret_id, String::new())
        }
    }
}

fn create_env_file(secrets: Vec<(String, String)>, output_file: &str) -> Result<()> {
    let mut file = match File::create(output_file) {
        Ok(f) => f,
        Err(err) => {
            error!("Failed to create output file: {}", err);
            return Err(err.into());
        }
    };

    for (key, value) in secrets {
        if let Some(secret_name) = key.split('/').last() {
            if let Err(err) = writeln!(file, "{}={}", secret_name, value) {
                error!("Failed to write to output file: {}: {}", output_file, err);
                return Err(err.into());
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::{self, BufRead};

    #[tokio::test]
    async fn test_create_env_file() -> Result<()> {
        let test_secrets = vec![
            ("SECRET_KEY".to_string(), "secret_value1".to_string()),
            ("API_KEY".to_string(), "secret_value2".to_string()),
        ];

        let test_file = "test_output.env";
        create_env_file(test_secrets, test_file)?;

        let file = fs::File::open(test_file)?;
        let reader = io::BufReader::new(file);
        let lines: Vec<String> = reader.lines().collect::<Result<_, _>>()?;

        assert_eq!(
            lines,
            vec!["SECRET_KEY=secret_value1", "API_KEY=secret_value2",]
        );

        fs::remove_file(test_file)?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    let mut log = Logger::new();

    let vault_url = format!("https://{}.vault.azure.net", opts.vault_name);

    log.loading("Detecting credentials.");
    let credential = DefaultAzureCredential::default();
    let client = match KeyvaultClient::new(&vault_url, std::sync::Arc::new(credential)) {
        Ok(c) => c,
        Err(err) => {
            error!("Failed to create KeyvaultClient: {}", err);
            return Err(err.into());
        }
    };
    log.success("Detected credentials.");

    check_vault_dns(&opts.vault_name).await?;

    log.loading(format!(
        "Fetching secrets from Key Vault: <blue>{}</>",
        opts.vault_name
    ));
    let secrets = fetch_secrets_from_key_vault(&client, opts.filter.as_deref()).await?;
    log.success(format!(
        "Fetched secrets from Key Vault: <blue>{}</>",
        opts.vault_name
    ));

    log.loading(format!("Creating output file: <blue>{}</>", opts.output));
    create_env_file(secrets, &opts.output)?;
    log.success(format!("Created output file: <blue>{}</>", opts.output));

    log.success("Done.");
    Ok(())
}
