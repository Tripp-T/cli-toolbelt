use crate::*;
use trust_dns_resolver::{
    config::{NameServerConfig, Protocol, ResolverConfig, ResolverOpts},
    error::ResolveErrorKind,
    proto::rr::RecordType,
    TokioAsyncResolver,
};

#[derive(Parser, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Opts {
    /// hosts to query
    #[clap(required = true)]
    fqdns: Vec<String>,
    /// record type to query
    #[clap(short = 't', long = "type", default_value = "A")]
    record_type: String,
    #[clap(short = '@', long)]
    name_server: Option<Vec<String>>,
}

pub async fn main(opts: &Opts) -> Result<()> {
    let r_config = if let Some(ns) = &opts.name_server {
        let mut cfg = ResolverConfig::new();
        for ns in ns.iter() {
            let addr = if ns.contains(':') {
                ns.to_owned()
            } else {
                format!("{}:53", ns)
            }
            .parse::<SocketAddr>()
            .map_err(|e| anyhow!("Failed to parse SocketAddr for name server '{ns}': {e}"))?;
            cfg.add_name_server(NameServerConfig::new(addr, Protocol::Udp))
        }
        cfg
    } else {
        ResolverConfig::default()
    };
    let r_opts = ResolverOpts::default();

    let record_type = RecordType::from_str(&opts.record_type.to_uppercase()).map_err(|e| {
        anyhow!(
            "Failed to parse record type from '{}': {e}",
            opts.record_type
        )
    })?;

    let resolver = TokioAsyncResolver::tokio(r_config.clone(), r_opts);

    for fqdn in opts.fqdns.iter() {
        println!("Query:\t\tFQDN: {};\tType: {};", fqdn, opts.record_type);

        let lookup = resolver.lookup(fqdn, record_type).await;

        let records = match lookup {
            Ok(lookup) => lookup.records().to_owned(),
            Err(e) => match e.kind() {
                ResolveErrorKind::NoRecordsFound { .. } => {
                    println!("Response:\nx\tNo records found!");
                    continue;
                }
                e => {
                    return Err(anyhow!("failed to query DNS: {e}"));
                }
            },
        };

        let mut response_spacing = [0; 2];
        for response in records.iter() {
            let str = response.to_string();
            let str = str.split(' ').collect::<Vec<_>>();
            if str.is_empty() {
                continue;
            }
            let str = str[0];
            if response.name().len() > response_spacing[0] {
                response_spacing[0] = response.name().len();
            }
            if str.len() > response_spacing[1] {
                response_spacing[1] = str.len();
            }
        }

        // print response
        println!("Response:\t{}", r_config.name_servers()[0]);
        // record header
        match record_type {
            RecordType::MX => println!(
                "[Query{}FQDN{}TTL\tDIR\tTYPE\tPRIO\tVALUE\t\t]",
                " ".repeat(response_spacing[0] - 5),
                " ".repeat(response_spacing[1])
            ),
            _ => println!(
                "[Query{}FQDN{}\tTTL\tDIR\tTYPE\tVALUE\t\t]",
                " ".repeat(response_spacing[0] - 5),
                " ".repeat(response_spacing[1] - 4)
            ),
        }

        for response in records.iter() {
            let str = response.to_string();
            let mut str = str.split(' ').map(|s| s.to_string()).collect::<Vec<_>>();
            str[0] = format!(
                "{}{}",
                str[0],
                " ".repeat(response_spacing[1] - str[0].len())
            );
            println!(
                "{}{} {}",
                response.name(),
                " ".repeat(response_spacing[0] - response.name().len()),
                str.join("\t")
            );
        }
    }
    Ok(())
}
