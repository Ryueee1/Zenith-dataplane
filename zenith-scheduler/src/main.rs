//! Zenith Job Scheduler - Main Entry Point

use clap::Parser;
use tracing::info;

#[derive(Parser)]
#[command(name = "zenith-scheduler")]
#[command(about = "Zenith GPU-aware Job Scheduler")]
struct Args {
    /// Configuration file path
    #[arg(short, long, default_value = "config.yaml")]
    config: String,
    
    /// gRPC listen address
    #[arg(long, default_value = "[::]:50051")]
    grpc_address: String,
    
    /// HTTP listen address
    #[arg(long, default_value = "0.0.0.0:8080")]
    http_address: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    
    info!("Starting Zenith Scheduler v{}", zenith_scheduler::VERSION);
    info!("gRPC: {}", args.grpc_address);
    info!("HTTP: {}", args.http_address);
    
    // In production: start gRPC and HTTP servers
    // For now, just wait
    tokio::signal::ctrl_c().await?;
    
    info!("Shutting down...");
    Ok(())
}
