use clap::Parser;
use tracing::info;

mod utils;

#[derive(Parser)]
#[command(name = "livevod", version)]
struct Args {
    /// Set config file path
    #[arg(short, long)]
    config: Option<String>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    
    // Load config
    let cfg: livevod::config::Config = utils::load("livevod".to_string(), args.config);
    cfg.validate().unwrap();

    info!("starting livevod with config: {:?}", cfg);

    let listener = tokio::net::TcpListener::bind(cfg.http.listen)
        .await
        .unwrap();

    livevod::serve(cfg, listener, utils::shutdown_signal()).await;
    info!("Server shutdown");
}
