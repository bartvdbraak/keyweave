use azure_identity::DefaultAzureCredential;
use azure_security_keyvault::KeyvaultClient;
use clap::Parser;
use futures::stream::StreamExt;
use std::fs::File;
use std::io::Write;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Opts {
    #[clap(
        short,
        long,
        value_name = "VAULT_NAME",
        help = "Sets the name of the Azure Key Vault"
    )]
    vault_name: String,

    #[clap(
        short,
        long,
        value_name = "FILE",
        default_value = ".env",
        help = "Sets the name of the output file"
    )]
    output: String,

    #[clap(
        short,
        long,
        value_name = "FILTER",
        help = "Filters the secrets to be retrieved by name"
    )]
    filter: Option<String>,
}

async fn fetch_secrets_from_key_vault(
    vault_url: &str,
    filter: Option<&str>,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let credential = DefaultAzureCredential::default();
    let client = KeyvaultClient::new(&vault_url, std::sync::Arc::new(credential))?.secret_client();

    let mut secret_values = Vec::new();
    let mut secret_pages = client.list_secrets().into_stream();

    while let Some(page) = secret_pages.next().await {
        let page = page?;
        for secret in &page.value {
            if let Some(filter) = filter {
                if !secret.id.contains(filter) {
                    continue;
                }
            }
            let secret_name = secret.id.split('/').last().unwrap_or_default();
            let secret_bundle = client.get(secret_name).await?;
            secret_values.push((secret.id.clone(), secret_bundle.value));
        }
    }

    Ok(secret_values)
}

fn create_env_file(secrets: Vec<(String, String)>, output_file: &str) -> std::io::Result<()> {
    let mut file = File::create(output_file)?;
    for (key, value) in secrets {
        // Extract the secret name from the URL
        if let Some(secret_name) = key.split('/').last() {
            writeln!(file, "{}={}", secret_name, value)?;
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts: Opts = Opts::parse();

    let vault_url = format!("https://{}.vault.azure.net", opts.vault_name);

    println!("Fetching secrets from Key Vault: {}", opts.vault_name);

    let secrets = fetch_secrets_from_key_vault(&vault_url, opts.filter.as_deref()).await?;

    println!("Creating output file: {}", opts.output);
    create_env_file(secrets, &opts.output)?;

    println!("Process completed successfully!");

    Ok(())
}
