use anyhow::{Context, Result};
use azure_identity::DefaultAzureCredential;
use azure_security_keyvault::prelude::KeyVaultGetSecretsResponse;
use azure_security_keyvault::KeyvaultClient;
use clap::Parser;
use futures::stream::StreamExt;
use paris::Logger;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Semaphore;
// use paris::error;

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

async fn fetch_secrets_from_key_vault(
    client: &KeyvaultClient,
    filter: Option<&str>,
) -> Result<Vec<(String, String)>> {
    let mut secret_values = Vec::new();
    let mut secret_pages = client.secret_client().list_secrets().into_stream();

    while let Some(page) = secret_pages.next().await {
        let page = page.context("Failed to fetch secrets page")?;
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
            eprintln!("Error fetching secret: {}", err);
            (secret_id, String::new())
        }
    }
}

fn create_env_file(secrets: Vec<(String, String)>, output_file: &str) -> Result<()> {
    let mut file = File::create(output_file).context("Failed to create output file")?;
    for (key, value) in secrets {
        if let Some(secret_name) = key.split('/').last() {
            writeln!(file, "{}={}", secret_name, value)
                .with_context(|| format!("Failed to write to output file: {}", output_file))?;
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

        fs::remove_file(test_file)?; // Clean up the test file
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
    let client = KeyvaultClient::new(&vault_url, std::sync::Arc::new(credential))
        .context("Failed to create KeyvaultClient")?;
    log.success("Detected credentials.");

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
