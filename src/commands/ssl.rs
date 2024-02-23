use std::{net::TcpStream, str::FromStr};

use anyhow::{anyhow, bail, Ok, Result};
use clap::Parser;
// use num_bigint::BigInt;
use openssl::{
    ssl::{SslConnector, SslMethod},
    string::OpensslString,
};
use url::Url;

#[derive(Parser, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Opts {
    /// hosts to query
    #[clap(required = true)]
    hosts: Vec<String>,
    /// allow insecure connections
    #[clap(short, long)]
    insecure: bool,
}

pub async fn main(opts: &Opts) -> Result<()> {
    let mut ssl_connector = SslConnector::builder(SslMethod::tls())?;
    if opts.insecure {
        ssl_connector.set_verify(openssl::ssl::SslVerifyMode::NONE);
        ssl_connector
            .set_min_proto_version(None)
            .map_err(|e| anyhow!("{}", e.to_string()))?;
    }
    let ssl_connector = ssl_connector.build();

    for (idx, input_host) in opts.hosts.iter().enumerate() {
        let mut host = input_host.to_owned();
        if !host.contains("://") {
            host = format!("https://{}", host);
        }
        let url = Url::from_str(&host)?;
        if url.scheme() != "https" {
            return Err(anyhow!(
                "only supported scheme is 'https://', received '{}'",
                url.scheme()
            ));
        }

        let host = url.host_str().ok_or(anyhow!("failed to get host"))?;
        let port = url
            .port_or_known_default()
            .ok_or(anyhow!("failed to get port"))?;
        let addr = format!("{}:{}", host, port);

        let stream = TcpStream::connect(&addr)
            .map_err(|e| anyhow!("failed to establish tcp connection to '{addr}': {e}"))?;
        let stream = ssl_connector
            .connect(host, stream)
            .map_err(|e| anyhow!("ssl error connecting to '{host}': {e}", e = e.to_string()))?;

        let chain = stream
            .ssl()
            .peer_cert_chain()
            .ok_or(anyhow!("failed to get peer certificate chain"))?;
        let cert = chain
            .get(0)
            .ok_or(anyhow!("failed to get peer certificate"))?;

        println!("Host:     {}", host);
        println!("Version:      {}", cert.version());
        println!(
            "Serial:       {}",
            cert.serial_number()
                .to_bn()
                .map_err(|e| anyhow!("failed to parse cert serial number: {e}"))?
                .to_hex_str()
                .map_err(|e| anyhow!("failed to parse serial number to hex: {e}"))
                .and_then(display_hex)?
        );
        println!("Not before:   {}", cert.not_before());
        println!("Not after:    {}", cert.not_after());
        println!("Subject:      {}", display_nameref(cert.subject_name())?);
        println!("Issuer:       {}", display_nameref(cert.issuer_name())?);

        if idx < opts.hosts.len() - 1 {
            println!();
        }
    }
    Ok(())
}

/// Display a nameref object's entries
fn display_nameref(nameref: &openssl::x509::X509NameRef) -> Result<String> {
    let mut output = String::new();
    for (idx, entry) in nameref.entries().enumerate() {
        if idx == 0 {
            output.push('\n');
        }
        output.push_str(&format!(
            "- {:?}: {:?}",
            entry.object(),
            entry.data().as_utf8()?
        ));
        if idx != nameref.entries().count() - 1 {
            output.push('\n');
        }
    }
    Ok(output)
}

/// Display a hex string with colons
fn display_hex(s: OpensslString) -> anyhow::Result<String> {
    if s.len() % 2 != 0 {
        bail!("invalid hex string '{s}': Hex is not even");
    }
    Ok(s.chars()
        .rev()
        .enumerate()
        .fold(String::new(), |mut acc, (idx, c)| {
            if idx != 0 && idx % 2 == 0 && idx != s.len() - 1 {
                // every 2 chars, except the last one
                acc.push(':');
            }
            acc.push(c);
            acc
        })
        .chars()
        .rev()
        .collect::<String>())
}
