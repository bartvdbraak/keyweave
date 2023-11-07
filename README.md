<h1 align="center">
    <img src="https://github.com/bartvdbraak/keyweave/assets/3996360/bed7f004-e897-46e5-98a4-c654251c0e17" alt="Cluster" width="336">
</h1>

# Keyweave

Keyweave is an open-source tool designed to seamlessly fetch secrets from Azure Key Vault and weave them into a convenient `.env` file. Developed in Rust, Keyweave is efficient and easy to use, making it an ideal choice for managing your application's secrets.

## Features

- **Fetch Secrets**: Retrieve secrets securely from Azure Key Vault.
- **Filtering**: Optionally filter the secrets to be retrieved by name.
- **Output Customization**: Choose the name of the output file, defaulting to `.env`.
- **Azure Default Credentials**: Utilizes Azure default credentials for authentication.

## Prerequisites

- **Rust**: Ensure you have Rust installed on your system. If not, you can install it using [rustup](https://rustup.rs/).
- **Azure Account**: Log into your Azure tenant and set up the right subscription.

## Installation

Clone the repository to your local machine:

```sh
git clone https://github.com/bartvdbraak/keyweave.git
cd keyweave
```

Build the project:

```sh
cargo build --release
```

## Usage

After building the project, you can run Keyweave using the following command:

```sh
cargo run -- --vault_name <VAULT_NAME> [--output <FILE>] [--filter <FILTER>]
```

- `--vault_name <VAULT_NAME>`: Sets the name of the Azure Key Vault.
- `--output <FILE>`: (Optional) Sets the name of the output file (default: `.env`).
- `--filter <FILTER>`: (Optional) Filters the secrets to be retrieved by name.

## Example

```sh
cargo run -- --vault_name my-key-vault --output my-env-file.env --filter my-secret
```

## License

Keyweave is licensed under the MIT License. See [LICENSE](LICENSE) for more details.

## Contributing

We welcome contributions! Please feel free to submit pull requests, report issues, or suggest new features.
