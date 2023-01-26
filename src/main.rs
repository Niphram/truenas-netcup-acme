use std::{env, fs};

use anyhow::Context;
use argh::FromArgs;
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

#[derive(Debug, FromArgs)]
/// this is the app
struct Cli {
    #[argh(subcommand)]
    /// test
    command: Commands,
}

#[derive(Debug, FromArgs)]
#[argh(subcommand)]
enum Commands {
    /// test2
    Set(SubCommandSet),
    /// test3
    Unset(SubCommandUnset),
}

#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "set")]
/// testsub
struct SubCommandSet {
    /// domain
    #[argh(positional)]
    domain: String,
    /// hostname
    #[argh(positional)]
    hostname: String,
    /// content
    #[argh(positional)]
    content: String,
}

#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "unset")]
/// testsub
struct SubCommandUnset {
    /// domain
    #[argh(positional)]
    domain: String,
    /// hostname
    #[argh(positional)]
    hostname: String,
    /// content
    #[argh(positional)]
    content: String,
}

#[async_std::main]
async fn main() -> anyhow::Result<()> {
    let mut config_path = env::current_exe()?;
    config_path.set_file_name("config.json");

    let args: Cli = argh::from_env();

    let contents = fs::read_to_string(&config_path)
        .context(format!("Failed to load {}", config_path.display()))?;
    let auth_args = serde_json::from_str::<NetcupAuth>(&contents)?;

    match args.command {
        Commands::Set(SubCommandSet {
            domain,
            hostname,
            content,
        }) => {
            let host = hostname
                .strip_suffix(&domain)
                .context("Hostname does not belong to domain!")?
                .strip_suffix('.')
                .context("Not a valid hostname")?;

            let client = NetcupAPIClient::login(
                auth_args.customer_id,
                auth_args.api_password,
                auth_args.api_key,
            )
            .await?;

            client.add_txt_record(&domain, host, &content).await?;

            client.logout().await
        }
        Commands::Unset(SubCommandUnset {
            domain,
            hostname,
            content,
        }) => {
            let host = hostname
                .strip_suffix(&domain)
                .context("Hostname does not belong to domain!")?
                .strip_suffix('.')
                .context("Not a valid hostname")?;

            let client = NetcupAPIClient::login(
                auth_args.customer_id,
                auth_args.api_password,
                auth_args.api_key,
            )
            .await?;

            let id = client.find_txt_record_id(&domain, host, &content).await?;
            client.delete_record(&id, &domain, host, &content).await?;

            client.logout().await
        }
    }
}
