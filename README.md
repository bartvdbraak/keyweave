# Keyweave

<img align="right" src="https://github.com/bartvdbraak/keyweave/assets/3996360/bed7f004-e897-46e5-98a4-c654251c0e17" alt="Cluster" height="256">

Keyweave is an open-source tool crafted to seamlessly fetch secrets from Azure Key Vault and weave them into a convenient `.env` file. Developed in Rust, Keyweave stands out for its efficiency and user-friendly design, making it an ideal choice for managing your application's secrets.

## Features

- **Fetch Secrets**: Retrieve secrets securely from Azure Key Vault.
- **Filtering**: Optionally filter the secrets to be retrieved by name.
- **Output Customization**: Choose the name of the output file, defaulting to `.env`.
- **Azure Default Credentials**: Utilizes Azure default credentials for authentication.

## Prerequisites

Before diving into Keyweave, ensure you have the following prerequisites:

- **Azure Account**: Log into your Azure tenant and set up the right subscription, along with any Access Policies required for you to read and list secrets from your Key Vault.

```sh
az login --tenant "your-tenant-guid"
az account set --subscription "your-subscription-guid"
```

## Installation (MacOS, Linux)

For MacOS and Linux systems, installation is a breeze with [Homebrew](https://brew.sh/). Simply run:

```bash
brew tap bartvdbraak/keyweave
brew install keyweave
```

## Manual Download 

If you prefer manual installation or need binaries for different platforms (including an executable for Windows), visit the [Releases](/releases) page of this GitHub repository.

## Building from Source

Keyweave is built with [Cargo](https://doc.rust-lang.org/cargo/), the Rust package manager.

To build Keyweave from source, follow these steps:

```sh
git clone https://github.com/bartvdbraak/keyweave.git
cd keyweave
cargo build --release
```

Once built, run Keyweave using Cargo:

```sh
cargo run -- --vault_name <VAULT_NAME> [--output <FILE>] [--filter <FILTER>]
```

## Usage

With the binary on your `PATH`, run Keyweave as follows:

```sh
keyweave --vault_name <VAULT_NAME> [--output <FILE>] [--filter <FILTER>]
```

- `--vault_name <VAULT_NAME>`: Sets the name of the Azure Key Vault.
- `--output <FILE>`: (Optional) Sets the name of the output file (default: `.env`).
- `--filter <FILTER>`: (Optional) Filters the secrets to be retrieved by name.

## Example

```sh
keyweave --vault_name my-key-vault --output my-env-file.env --filter my-secret
```

## License

Keyweave is licensed under the GPLv3 License. See [LICENSE](LICENSE) for more details.

## Contributing

We welcome contributions! Feel free to submit pull requests, report issues, or suggest new features. Your input helps make Keyweave even better.
