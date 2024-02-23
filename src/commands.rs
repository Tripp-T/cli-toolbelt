/*

    This is a mapper for all commands

*/

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
