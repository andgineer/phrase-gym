use std::net::{Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use clap::{Parser, Subcommand};
use phrase_gym_contracts::connection::DEFAULT_SERVER_PORT;
use phrase_gym_core::clock::SystemClock;
use phrase_gym_core::media::store::MediaStore;
use phrase_gym_core::recovery;
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
        #[arg(long, default_value_t = DEFAULT_SERVER_PORT)]
        port: u16,
        /// Built client directory containing index.html.
        #[arg(long)]
        client_dir: Option<PathBuf>,
    },
    /// Write the API schema without opening any database.
    ExportSchema { output: PathBuf },
    /// Take a consistent database and referenced-media snapshot, including live WAL writes.
    Backup {
        #[arg(long)]
        data_dir: PathBuf,
        #[arg(long)]
        snapshot: PathBuf,
    },
    /// Verify a complete snapshot without modifying it or opening a live data directory.
    VerifyBackup {
        #[arg(long)]
        snapshot: PathBuf,
    },
    /// Restore a complete matching snapshot into a new isolated data directory.
    Restore {
        #[arg(long)]
        snapshot: PathBuf,
        #[arg(long)]
        data_dir: PathBuf,
    },
}

fn main() -> anyhow::Result<()> {
    match Cli::parse().command {
        Command::Serve {
            data_dir,
            port,
            client_dir,
        } => {
            let settings = ServerSettings {
                client_dir,
                ..ServerSettings::loopback()
            };
            serve(&data_dir, port, &settings)
        }
        Command::ExportSchema { output } => export_schema(&output),
        Command::Backup { data_dir, snapshot } => {
            let result = recovery::backup(&data_dir, &snapshot)?;
            println!(
                "Backup complete: Rust schema {} ({})",
                result.schema_revision,
                snapshot.display()
            );
            Ok(())
        }
        Command::VerifyBackup { snapshot } => {
            let result = recovery::verify(&snapshot)?;
            println!(
                "Backup verified: Rust schema {}, {} media files",
                result.schema_revision,
                result.media.len()
            );
            Ok(())
        }
        Command::Restore { snapshot, data_dir } => {
            let result = recovery::restore(&snapshot, &data_dir)?;
            println!(
                "Restore complete: Rust schema {} ({})",
                result.schema_revision,
                data_dir.display()
            );
            Ok(())
        }
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
    let database = Arc::new(database);
    phrase_gym_core::profiles::Profiles::new(database.clone(), Arc::new(SystemClock))
        .initialize_current_learner()?;
    let state = AppState::new(database, Arc::new(store), Arc::new(SystemClock));
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;
    runtime.block_on(async move {
        let address = SocketAddr::from((Ipv4Addr::LOCALHOST, port));
        let listener = tokio::net::TcpListener::bind(address)
            .await
            .with_context(|| format!("binding {address}"))?;
        tracing::info!("serving http://{}", listener.local_addr()?);
        axum::serve(
            listener,
            router(state, settings).into_make_service_with_connect_info::<SocketAddr>(),
        )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_launch_uses_the_shared_default_and_only_an_explicit_port_override() {
        for (options, expected) in [
            (vec![], DEFAULT_SERVER_PORT),
            (vec!["--port", "8181"], 8181),
        ] {
            let mut arguments = vec!["phrase-gym-server", "serve", "--data-dir", "/tmp/synthetic"];
            arguments.extend(options);
            let Command::Serve { port, .. } = Cli::try_parse_from(arguments).unwrap().command
            else {
                panic!("Expected local serve command");
            };
            assert_eq!(port, expected);
        }
        assert!(
            Cli::try_parse_from([
                "phrase-gym-server",
                "serve",
                "--data-dir",
                "/tmp/synthetic",
                "--tailscale-origin",
                "https://gym.example.ts.net"
            ])
            .is_err()
        );
    }
}
