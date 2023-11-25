use assert_cmd::prelude::*;
use assert_fs::prelude::*;
use assert_fs::TempDir;
use std::process::Command;
use predicates::prelude::*;
use serial_test::serial;
use std::env;

static BINARY: &str = "keyweave";
static KEYVAULT: &str = "bvdbkeyweavetweukvt1";
static FIREWALL_KEYVAULT: &str = "bvdbkeyweavetweukvt2";
static NON_EXISTENT_KEYVAULT: &str = "bvdbkeyweavetweukvt3";

fn azure_cli_login(client_id: String, tenant_id: String, subscription_id: String) -> Result<(), std::io::Error> {
    println!("Executing 'az login' with client ID: {}", client_id);
    let login_output = Command::new("az")
        .arg("login")
        .arg("--identity")
        .arg("--username")
        .arg(&client_id)
        .arg("--tenant")
        .arg(&tenant_id)
        .output()?;
    println!("Login output: {:?}", login_output);
    println!("Executing 'az account set' with subscription ID: {}", subscription_id);
    let account_output = Command::new("az")
        .arg("account")
        .arg("set")
        .arg("--subscription")
        .arg(&subscription_id)
        .output()?;
    println!("Account output: {:?}", account_output);
    Ok(())
}

/// Test with no access policies - expected to fail.
#[tokio::test]
#[serial]
async fn test_no_access_policies() {
    azure_cli_login(
        env::var("AZURE_CLIENT_ID_NO_ACCESS").expect("Failed to get AZURE_CLIENT_ID_NO_ACCESS"),
        env::var("AZURE_TENANT_ID").expect("Failed to get AZURE_TENANT_ID"),
        env::var("AZURE_SUBSCRIPTION_ID").expect("Failed to get AZURE_SUBSCRIPTION_ID"),
    ).expect("Failed to log in to Azure CLI");

    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.child(".env");

    let mut cmd = Command::cargo_bin(BINARY).unwrap();
    cmd.arg("--vault-name").arg(KEYVAULT)
       .arg("--output").arg(output_path.path());
    cmd.assert().failure().stderr(predicate::str::contains("Make sure you have List permissions on the Key Vault."));

    temp_dir.close().unwrap();
}

/// Test with only Get access policy - expected to fail.
#[tokio::test]
#[serial]
async fn test_only_get_access_policy() {
    azure_cli_login(
        env::var("AZURE_CLIENT_ID_GET").expect("Failed to get AZURE_CLIENT_ID_GET"),
        env::var("AZURE_TENANT_ID").expect("Failed to get AZURE_TENANT_ID"),
        env::var("AZURE_SUBSCRIPTION_ID").expect("Failed to get AZURE_SUBSCRIPTION_ID"),
    ).expect("Failed to log in to Azure CLI");

    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.child(".env");

    let mut cmd = Command::cargo_bin(BINARY).unwrap();
    cmd.arg("--vault-name").arg(KEYVAULT)
       .arg("--output").arg(output_path.path());
    cmd.assert().failure().stderr(predicate::str::contains("Make sure you have List permissions on the Key Vault."));

    temp_dir.close().unwrap();
}

/// Test with only List access policy - expected to succeed with get errors.
#[tokio::test]
#[serial]
async fn test_only_list_access_policy() {
    azure_cli_login(
        env::var("AZURE_CLIENT_ID_LIST").expect("Failed to get AZURE_CLIENT_ID_LIST"),
        env::var("AZURE_TENANT_ID").expect("Failed to get AZURE_TENANT_ID"),
        env::var("AZURE_SUBSCRIPTION_ID").expect("Failed to get AZURE_SUBSCRIPTION_ID"),
    ).expect("Failed to log in to Azure CLI");

    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.child(".env");

    let mut cmd = Command::cargo_bin(BINARY).unwrap();
    cmd.arg("--vault-name").arg(KEYVAULT)
       .arg("--output").arg(output_path.path());
    cmd.assert().success().stderr(predicate::str::contains("Make sure you have Get permissions on the Key Vault."));

    temp_dir.close().unwrap();
}

/// Test with both Get and List access policies - expected to pass.
#[tokio::test]
#[serial]
async fn test_get_and_list_access_policies() {
    azure_cli_login(
        env::var("AZURE_CLIENT_ID_GET_LIST").expect("Failed to get AZURE_CLIENT_ID_GET_LIST"),
        env::var("AZURE_TENANT_ID").expect("Failed to get AZURE_TENANT_ID"),
        env::var("AZURE_SUBSCRIPTION_ID").expect("Failed to get AZURE_SUBSCRIPTION_ID"),
    ).expect("Failed to log in to Azure CLI");

    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.child(".env");

    let mut cmd = Command::cargo_bin(BINARY).unwrap();
    cmd.arg("--vault-name").arg(KEYVAULT)
       .arg("--output").arg(output_path.path());
    cmd.assert().success();

    output_path.assert(predicate::path::is_file());
    output_path.assert(predicate::str::contains("testSecret=testSecretValue"));
    output_path.assert(predicate::str::contains("filterTestSecret=filterTestSecretValue"));

    temp_dir.close().unwrap();
}

/// Test with both Get and List access policies and filter - expected to pass.
#[tokio::test]
#[serial]
async fn test_get_and_list_access_policies_filter() {
    azure_cli_login(
        env::var("AZURE_CLIENT_ID_GET_LIST").expect("Failed to get AZURE_CLIENT_ID_GET_LIST"),
        env::var("AZURE_TENANT_ID").expect("Failed to get AZURE_TENANT_ID"),
        env::var("AZURE_SUBSCRIPTION_ID").expect("Failed to get AZURE_SUBSCRIPTION_ID"),
    ).expect("Failed to log in to Azure CLI");

    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.child(".env");

    let mut cmd = Command::cargo_bin(BINARY).unwrap();
    cmd.arg("--vault-name").arg(KEYVAULT)
       .arg("--output").arg(output_path.path())
       .arg("--filter").arg("filter");
    cmd.assert().success();

    output_path.assert(predicate::path::is_file());
    output_path.assert(predicate::str::contains("filterTestSecret=filterTestSecretValue"));

    temp_dir.close().unwrap();
}

/// Test with both Get and List access policies on a Key Vault with Firewall - expected to fail.
#[tokio::test]
#[serial]
async fn test_get_and_list_access_policies_firewall() {
    azure_cli_login(
        env::var("AZURE_CLIENT_ID_GET_LIST").expect("Failed to get AZURE_CLIENT_ID_GET_LIST"),
        env::var("AZURE_TENANT_ID").expect("Failed to get AZURE_TENANT_ID"),
        env::var("AZURE_SUBSCRIPTION_ID").expect("Failed to get AZURE_SUBSCRIPTION_ID"),
    ).expect("Failed to log in to Azure CLI");

    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.child(".env");

    let mut cmd = Command::cargo_bin(BINARY).unwrap();
    cmd.arg("--vault-name").arg(FIREWALL_KEYVAULT)
       .arg("--output").arg(output_path.path());
    cmd.assert().failure().stderr(predicate::str::contains("Make sure you're on the Key Vaults Firewall allowlist."));

    temp_dir.close().unwrap();
}

/// Test with both Get and List access policies on a non-existent Key Vault - expected to fail.
#[tokio::test]
#[serial]
async fn test_get_and_list_access_policies_non_existent() {
    azure_cli_login(
        env::var("AZURE_CLIENT_ID_GET_LIST").expect("Failed to get AZURE_CLIENT_ID_GET_LIST"),
        env::var("AZURE_TENANT_ID").expect("Failed to get AZURE_TENANT_ID"),
        env::var("AZURE_SUBSCRIPTION_ID").expect("Failed to get AZURE_SUBSCRIPTION_ID"),
    ).expect("Failed to log in to Azure CLI");

    let temp_dir = TempDir::new().unwrap();
    let output_path = temp_dir.child(".env");

    let mut cmd = Command::cargo_bin(BINARY).unwrap();
    cmd.arg("--vault-name").arg(NON_EXISTENT_KEYVAULT)
       .arg("--output").arg(output_path.path());
    cmd.assert().failure().stderr(predicate::str::contains("Please check that the Key Vault exists or that you have no connectivity issues."));

    temp_dir.close().unwrap();
}

