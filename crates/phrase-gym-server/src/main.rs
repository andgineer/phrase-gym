use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use clap::{Parser, Subcommand};
use phrase_gym_core::clock::SystemClock;
use phrase_gym_core::storage::Database;
use phrase_gym_server::{AppState, openapi, router};
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
#[command(name = "phrase-gym-server", version, about = "Phrase Gym Server")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Serve the HTTP API on loopback using an explicit data directory.
    Serve {
        #[arg(long)]
        data_dir: PathBuf,
        #[arg(long, default_value_t = 8000)]
        port: u16,
    },
    /// Write the API schema without opening any database.
    ExportSchema { output: PathBuf },
}

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Command::Serve { data_dir, port } => serve(&data_dir, port),
        Command::ExportSchema { output } => export_schema(&output),
    }
}

fn serve(data_dir: &std::path::Path, port: u16) -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();
    let database = Database::open(data_dir)
        .with_context(|| format!("opening database in {}", data_dir.display()))?;
    let state = AppState::new(Arc::new(database), Arc::new(SystemClock));
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async move {
        let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
        let listener = tokio::net::TcpListener::bind(address)
            .await
            .with_context(|| format!("binding {address}"))?;
        tracing::info!("serving http://{}", listener.local_addr()?);
        axum::serve(listener, router(state))
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
