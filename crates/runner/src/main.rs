use std::{fs, path::PathBuf};

use clap::Parser;

/// simdash simulation runner
#[derive(Parser)]
struct Cli {
    /// Path to the JSON config file
    #[arg(default_value = "config.json")]
    config: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    let contents = fs::read_to_string(&cli.config)
        .unwrap_or_else(|e| panic!("Failed to read '{}': {e}", cli.config.display()));

    let config: serde_json::Value = serde_json::from_str(&contents)
        .unwrap_or_else(|e| panic!("Failed to parse '{}' as JSON: {e}", cli.config.display()));

    println!("Hello, world from simdash runner!");
    println!("Config loaded from '{}':", cli.config.display());
    println!("{config:#}");
}
