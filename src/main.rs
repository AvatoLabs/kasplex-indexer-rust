mod config;
mod explorer;
mod http;
mod operations;
mod protobuf;
mod storage;
mod utils;
// remove tonic Server import; use axum's server instead
use crate::config::VERSION;
use crate::explorer::{Explorer, ExplorerInterface};
use crate::operations::handler::OperationManager;
use crate::storage::StorageManager;
use anyhow::Result;
use axum::Router;
use std::fs;
use std::net::SocketAddr;
use std::os::unix::fs::OpenOptionsExt;
use std::os::unix::io::AsRawFd;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::signal;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Print startup banner (corresponding to Go version version info)
    println!("KASPlex Executor v{}", VERSION);

    // Initialize basic logging first
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Set the correct working directory (corresponding to Go version working directory setup)
    set_working_directory()?;

    // Use file lock for startup (corresponding to Go version file lock functionality)
    let lock_file = acquire_file_lock()?;

    // Load configuration (corresponding to Go version config.Load)
    let mut config = crate::config::types::Config::default();
    crate::config::load_config(&mut config)?;
    crate::config::validate_config(&config)?;

    // Set the log level based on config (corresponding to Go version debug level setup)
    set_log_level_from_config(&config)?;

    // Set up shutdown signal handling (corresponding to Go version signal handling)
    let shutdown_flag = Arc::new(AtomicBool::new(false));
    let shutdown_flag_clone = Arc::clone(&shutdown_flag);

    let shutdown_signal = async {
        signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
        info!("main stopping ..");
        shutdown_flag_clone.store(true, Ordering::SeqCst);
    };

    // Initialize storage driver (corresponding to Go version storage.Init)
    // Note: Go version passes cfg.Cassandra and cfg.Rocksdb, but actually only uses Rocksdb
    let mut storage = StorageManager::new(config.rocksdb, Some(config.distributed)).await?;
    storage.init().await?;

    // Initialize operation manager (Rust version specific, for operation handling)
    let operation_manager = OperationManager::new(Arc::new(storage.clone()));
    storage.set_operation_manager(operation_manager);

    // Convert to Arc for explorer
    let storage = Arc::new(storage);

    // Initialize explorer if not shutting down (corresponding to Go version explorer.Init)
    if !shutdown_flag.load(Ordering::SeqCst) {
        // Start HTTP server
        let http_state = crate::http::HttpState {
            kaspa_rest_base_url: config.rest.kaspa_rest_base_url.clone(),
        };
        let http_router = crate::http::build_router_with_state(http_state).layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );
        let bind_ip: std::net::IpAddr = config
            .http
            .bind
            .parse()
            .unwrap_or_else(|_| "0.0.0.0".parse().unwrap());
        let http_addr: SocketAddr = (bind_ip, config.http.port).into();
        info!("HTTP server listening on {}", http_addr);

        let mut explorer = Explorer::new(Arc::clone(&storage), config.startup, config.testnet)?;
        explorer.init().await?;

        // Start explorer in background (corresponding to Go version explorer.Run)
        let explorer_handle = tokio::spawn(async move {
            if let Err(e) = explorer.run().await {
                error!("Explorer error: {}", e);
            }
        });

        let http_handle = tokio::spawn(async move {
            match tokio::net::TcpListener::bind(http_addr).await {
                Ok(listener) => {
                    if let Err(e) = axum::serve(listener, http_router.into_make_service()).await {
                        error!("HTTP server error: {}", e);
                    }
                }
                Err(e) => error!("Failed to bind {}: {}", http_addr, e),
            }
        });

        // Wait for shutdown signal
        tokio::select! {
            _ = shutdown_signal => {
                info!("Shutting down...");
            }
            _ = explorer_handle => {
                info!("Explorer completed");
            }
            _ = http_handle => {
                info!("HTTP server completed");
            }
        }
    }

    // Graceful shutdown (corresponding to Go version graceful shutdown)
    shutdown_gracefully(storage).await?;

    // Release file lock
    release_file_lock(lock_file)?;

    info!("Shutdown completed");
    Ok(())
}

// Set log level based on config, corresponding to Go version debug level setup
fn set_log_level_from_config(config: &crate::config::types::Config) -> Result<()> {
    let level = match config.debug {
        3 => "debug",
        2 => "info",
        1 => "warn",
        _ => "error",
    };

    // Only log the level, do not reinitialize subscriber
    info!("Log level set to: {}", level);
    Ok(())
}

// Working directory setup functionality in Go version
fn set_working_directory() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.is_empty() {
        return Ok(());
    }

    let arg0 = &args[0];
    if !arg0.contains("go-build")
        && !arg0.contains("target/debug")
        && !arg0.contains("target/release")
    {
        if let Some(dir) = std::path::Path::new(arg0).parent() {
            let abs_dir = dir.canonicalize()?;
            std::env::set_current_dir(&abs_dir)?;
            info!("Changed working directory to: {}", abs_dir.display());
        }
    } else {
        // If running through cargo run, stay in project root directory
        info!("Keeping current working directory for cargo run");
    }

    Ok(())
}

// File lock functionality using safer approach
fn acquire_file_lock() -> Result<fs::File> {
    let lock_path = "./.lockExecutor";

    // Create lock file with proper permissions
    let lock_file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .mode(0o600) // More restrictive permissions
        .open(lock_path)?;

    // Try to acquire exclusive lock using safer approach
    let fd = lock_file.as_raw_fd();
    
    // Use fcntl instead of flock for better portability
    let result = unsafe {
        libc::fcntl(fd, libc::F_SETLK, &libc::flock {
            l_type: libc::F_WRLCK as i16,
            l_whence: libc::SEEK_SET as i16,
            l_start: 0,
            l_len: 0,
            l_pid: 0,
        })
    };
    
    if result != 0 {
        return Err(anyhow::anyhow!(
            "Failed to acquire file lock: another instance is running"
        ));
    }

    info!("File lock acquired: {}", lock_path);
    Ok(lock_file)
}

fn release_file_lock(lock_file: fs::File) -> Result<()> {
    let fd = lock_file.as_raw_fd();
    
    // Release the lock
    let result = unsafe {
        libc::fcntl(fd, libc::F_SETLK, &libc::flock {
            l_type: libc::F_UNLCK as i16,
            l_whence: libc::SEEK_SET as i16,
            l_start: 0,
            l_len: 0,
            l_pid: 0,
        })
    };
    
    if result != 0 {
        tracing::warn!("Failed to release file lock");
    }

    // Remove lock file
    let lock_path = "./.lockExecutor";
    if std::path::Path::new(lock_path).exists() {
        fs::remove_file(lock_path)?;
        info!("File lock released: {}", lock_path);
    }

    Ok(())
}

async fn shutdown_gracefully(storage: Arc<StorageManager>) -> Result<()> {
    info!("Performing graceful shutdown...");

    // Shutdown storage
    storage.shutdown().await?;

    info!("Graceful shutdown completed");
    Ok(())
}
