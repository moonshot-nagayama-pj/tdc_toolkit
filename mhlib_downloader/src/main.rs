use clap::Parser;
use mhlib_downloader::{download, validate_hash, DEFAULT_MHLIB_VERSION};

#[derive(Parser)]
struct Args {
    #[arg(default_value_t = DEFAULT_MHLIB_VERSION.to_string())]
    mhlib_version: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    println!("mhlib_version: {}", args.mhlib_version);
    validate_hash();
    download().await;
}
