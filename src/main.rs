use std::{env, fs};

use anyhow::Context;
use clap::{Parser, Subcommand};
use serde::Deserialize;
use truenas_acme_auth::NetcupAPIClient;

#[derive(Debug, Deserialize)]
struct NetcupAuth {
    #[serde(rename = "CID")]
    customer_id: String,
    #[serde(rename = "API_PW")]
    api_password: String,
    #[serde(rename = "API_KEY")]
    api_key: String,
}

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Set {
        domain: String,
        hostname: String,
        content: String,
    },
    Unset {
        domain: String,
        hostname: String,
        content: String,
    },
}

fn main() -> anyhow::Result<()> {
    let mut config_path = env::current_exe()?;
    config_path.set_file_name("config.toml");

    let args = Cli::parse();

    let contents = fs::read_to_string(&config_path)
        .context(format!("Failed to load {}", config_path.display()))?;
    let auth_args = toml::from_str::<NetcupAuth>(&contents)?;

    let client = NetcupAPIClient::login(
        auth_args.customer_id,
        auth_args.api_password,
        auth_args.api_key,
    )?;

    match args.command {
        Commands::Set {
            domain,
            hostname,
            content,
        } => {
            let host = hostname
                .strip_suffix(&domain)
                .context("Hostname does not belong to domain!")?
                .strip_suffix('.')
                .context("Not a valid hostname")?;

            client.add_txt_record(&domain, host, &content)
        }
        Commands::Unset {
            domain,
            hostname,
            content,
        } => {
            let host = hostname
                .strip_suffix(&domain)
                .context("Hostname does not belong to domain!")?
                .strip_suffix('.')
                .context("Not a valid hostname")?;

            let id = client.find_txt_record_id(&domain, host, &content)?;
            client.delete_record(&id, &domain, host, &content)
        }
    }
}
