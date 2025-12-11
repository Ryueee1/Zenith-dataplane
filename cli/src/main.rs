use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::fs;
use serde::Deserialize;

#[derive(Parser)]
#[command(name = "zenith")]
#[command(about = "Zenith Data Plane CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the Zenith Engine
    Start {
        /// Path to configuration file
        #[arg(short, long, default_value = "config/zenith.toml")]
        config: PathBuf,
    },
    /// Show version
    Version,
}

#[derive(Deserialize)]
struct Config {
    server: ServerConfig,
    engine: EngineConfig,
}

#[derive(Deserialize)]
struct ServerConfig {
    port: u16,
}

#[derive(Deserialize)]
struct EngineConfig {
    buffer_size: usize,
    plugins: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { config } => {
            println!("Starting Zenith Engine...");
            
            // Read Config
            let config_content = fs::read_to_string(&config)
                .unwrap_or_else(|_| "
[server]
port = 8080

[engine]
buffer_size = 1024
plugins = []
".to_string());
            
            let cfg: Config = toml::from_str(&config_content)?;
            
            println!("Config loaded: buffer_size={}, port={}", cfg.engine.buffer_size, cfg.server.port);

            // Init Engine
            // Note: In a real CLI, we might want to attach signals to shutdown cleanly
            let engine = zenith_core::Engine::new(cfg.engine.buffer_size)?;
            
            // Load Plugins
            for plugin_path in cfg.engine.plugins {
                 println!("Loading plugin: {}", plugin_path);
                 let wasm_bytes = fs::read(&plugin_path)?;
                 engine.load_plugin(&wasm_bytes)?;
            }

            engine.start();
            println!("Engine started. Admin API at http://localhost:{}", cfg.server.port);

            // Block forever
            std::thread::park();
        }
        Commands::Version => {
            println!("Zenith Data Plane v0.1.0");
        }
    }

    Ok(())
}
