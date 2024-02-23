#![allow(dead_code)]
use anyhow::Result;
use clap::Parser;

mod commands;
mod models;
mod utils;

/// A CLI tool for IT related tasks.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Opts {
    /// The command to run
    #[clap(subcommand)]
    cmd: Box<commands::Command>,
}

#[tokio::main]
async fn main() -> Result<()> {
    std::env::var("RUST_LOG")
        .map_err(|e| {
            if e == std::env::VarError::NotPresent {
                std::env::set_var("RUST_LOG", "info");
            }
        })
        .ok();
    env_logger::init();
    Opts::parse().cmd.run().await
}
