use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use clap::{Parser, Subcommand};
use phrase_gym_core::clock::SystemClock;
use phrase_gym_core::media::store::MediaStore;
use phrase_gym_core::storage::Database;
use phrase_gym_server::{AppState, ServerSettings, openapi, router};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "phrase-gym-server", version, about = "Phrase Gym Server")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Serve the HTTP API and built client on loopback using an explicit data directory.
    Serve {
        #[arg(long)]
        data_dir: PathBuf,
        #[arg(long, default_value_t = 8000)]
        port: u16,
        /// Built client directory containing index.html.
        #[arg(long)]
        client_dir: Option<PathBuf>,
        /// Additional trusted Host name, such as a private HTTPS proxy name; repeatable.
        #[arg(long = "allowed-host")]
        allowed_hosts: Vec<String>,
        /// Origin allowed to send mutations, such as `https://phrase-gym.example`; repeatable.
        #[arg(long = "allowed-origin")]
        allowed_origins: Vec<String>,
    },
    /// Write the API schema without opening any database.
    ExportSchema { output: PathBuf },
}

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Command::Serve {
            data_dir,
            port,
            client_dir,
            allowed_hosts,
            allowed_origins,
        } => {
            let settings = ServerSettings {
                allowed_origins,
                client_dir,
                ..ServerSettings::loopback()
            }
            .with_hosts(allowed_hosts);
            serve(&data_dir, port, &settings)
        }
        Command::ExportSchema { output } => export_schema(&output),
    }
}

fn serve(data_dir: &std::path::Path, port: u16, settings: &ServerSettings) -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    let database = Database::open(data_dir)
        .with_context(|| format!("opening database in {}", data_dir.display()))?;
    let store = MediaStore::open(data_dir)
        .with_context(|| format!("opening media storage in {}", data_dir.display()))?;
    let state = AppState::new(Arc::new(database), Arc::new(store), Arc::new(SystemClock));
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async move {
        let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
        let listener = tokio::net::TcpListener::bind(address)
            .await
            .with_context(|| format!("binding {address}"))?;
        tracing::info!("serving http://{}", listener.local_addr()?);
        axum::serve(listener, router(state, settings))
            .with_graceful_shutdown(async {
                let _ = tokio::signal::ctrl_c().await;
            })
            .await
            .context("serving HTTP")
    })
}

fn export_schema(output: &std::path::Path) -> anyhow::Result<()> {
    if let Some(parent) = output
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(output, openapi::document_json())
        .with_context(|| format!("writing {}", output.display()))
}
