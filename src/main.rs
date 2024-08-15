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

mod inputs;
use inputs::prelude::*;
mod clients;
use clients::prelude::*;
mod commands {
    mod cidr;
    mod diff;
    mod dns;
    mod domain;
    mod ports;
    mod rng;
    mod ssl;
    mod tcp;

    #[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, clap::Subcommand)]
    pub enum Command {
        /// Calculate and display info on CIDR
        Cidr(cidr::Opts),
        /// Query DNS records
        Dns(dns::Opts),
        /// Establish a SSL TCP connection and display SSL certificate
        Ssl(ssl::Opts),
        /// Generate a random number from a range
        #[clap(alias = "roll")]
        Rng(rng::Opts),
        /// Create a TCP connection (like telnet)
        Tcp(tcp::Opts),
        /// Scan host(s) TCP port(s)
        Ports(ports::Opts),
        /// Generate an overview of a domain
        Domain(domain::Opts),
        /// Compare the lines of two files
        Diff(diff::Opts),
    }

    impl Command {
        pub async fn run(&mut self) -> anyhow::Result<()> {
            match self {
                Command::Cidr(opts) => cidr::main(opts).await,
                Command::Dns(opts) => dns::main(opts).await,
                Command::Ssl(opts) => ssl::main(opts).await,
                Command::Rng(opts) => rng::main(opts).await,
                Command::Tcp(opts) => tcp::main(opts).await,
                Command::Ports(opts) => ports::main(opts).await,
                Command::Domain(opts) => domain::main(opts).await,
                Command::Diff(opts) => diff::main(opts).await,
            }
        }
    }
}
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
