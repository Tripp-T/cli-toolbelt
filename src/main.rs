#![allow(dead_code, unused_imports)]
use ::{
    anyhow::{anyhow, bail, ensure, Context, Result},
    clap::Parser,
    colored::Colorize,
    rand::Rng,
    serde::{Deserialize, Serialize},
    std::{
        net::{IpAddr, SocketAddr, TcpStream},
        path::PathBuf,
        process::exit,
        str::FromStr,
        sync::Arc,
    },
    tokio::io::{AsyncReadExt, AsyncWriteExt},
    tracing::{debug, error, info, level_filters::LevelFilter, trace, warn},
    tracing_subscriber::EnvFilter,
    url::Url,
};

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
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env()?,
        )
        .init();
    Opts::parse().cmd.run().await
}
