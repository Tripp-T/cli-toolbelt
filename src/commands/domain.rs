use crate::{
    models::{domain::DomainValidator, Domain},
    *,
};
use trust_dns_resolver::TokioAsyncResolver;

#[derive(clap::Parser, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Opts {
    #[clap(required = true)]
    domains: Vec<Domain>,
}

pub async fn main(opts: &mut Opts) -> anyhow::Result<()> {
    let validator = Arc::new(DomainValidator::new()?);

    let mut tasks = Vec::with_capacity(opts.domains.len());
    for domain in opts.domains.drain(..) {
        let validator = validator.clone();
        tasks.push(Box::pin(async move {
            validator.validate(&domain).await?;
            DomainReport::generate(domain).await
        }));
    }

    for (idx, task) in futures::future::join_all(tasks.into_iter().map(tokio::spawn))
        .await
        .iter()
        .enumerate()
    {
        if idx > 0 {
            println!();
        }
        match task {
            Ok(Ok(report)) => println!("{}", report),
            Ok(Err(e)) => println!("{}", e),
            Err(e) => println!("thread error: {}", e),
        }
    }

    Ok(())
}

struct MxRecord {
    host: String,
    priority: u16,
}

struct SoaRecord {
    mname: String,
    rname: String,
    serial: u32,
    refresh: u32,
    retry: u32,
    expire: u32,
    minimum: u32,
}

struct DomainReport {
    domain: Domain,
    ns: Option<Vec<String>>,
    mx: Option<Vec<MxRecord>>,
    spf: Option<String>,
    txt: Option<Vec<String>>,
    soa: Option<Vec<SoaRecord>>,
}

impl std::fmt::Display for DomainReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Domain: {}", self.domain)?;

        match self.ns {
            Some(ref ns) => {
                writeln!(f, "- NS:")?;
                for ns in ns.iter() {
                    writeln!(f, "\t\"{}\"", ns)?;
                }
            }
            None => writeln!(f, "- NS:\t<none>")?,
        }

        match self.mx {
            Some(ref mx) => {
                writeln!(f, "- MX:")?;
                for mx in mx.iter() {
                    writeln!(f, "\t(priority: {})\t\"{}\"", mx.priority, mx.host)?;
                }
            }
            None => writeln!(f, "- MX:\t<none>")?,
        }

        match self.spf {
            Some(ref spf) => writeln!(f, "- SPF:\t\"{}\"", spf)?,
            None => writeln!(f, "- SPF:\t<none>")?,
        }

        match self.txt {
            Some(ref txt) => {
                writeln!(f, "- TXT:")?;
                for txt in txt.iter() {
                    writeln!(f, "\t\"{txt}\"")?;
                }
            }
            None => writeln!(f, "- TXT:\t<none>")?,
        }

        match self.soa {
            Some(ref soa) => {
                writeln!(f, "- SOA:")?;
                for (idx, soa) in soa.iter().enumerate() {
                    if idx > 0 {
                        writeln!(f)?;
                    }
                    writeln!(f, "\t\"{}\"\t\"{}\"", soa.mname, soa.rname)?;
                    writeln!(f, "\t\tserial: {}", soa.serial)?;
                    writeln!(f, "\t\trefresh: {}", soa.refresh)?;
                    writeln!(f, "\t\tretry: {}", soa.retry)?;
                    writeln!(f, "\t\texpire: {}", soa.expire)?;
                    writeln!(f, "\t\tminimum: {}", soa.minimum)?;
                }
            }
            None => writeln!(f, "- SOA:\t<none>")?,
        }

        Ok(())
    }
}

impl DomainReport {
    pub async fn generate(domain: Domain) -> anyhow::Result<Self> {
        let dns_client = TokioAsyncResolver::tokio_from_system_conf()
            .map_err(|e| anyhow!("failed to create DNS resolver: {e}"))?;
        let domain_str = domain.as_str();

        let ns = dns_client
            .ns_lookup(domain_str)
            .await
            .map_or_else(
                |e| match *e.kind() {
                    trust_dns_resolver::error::ResolveErrorKind::NoRecordsFound { .. } => {
                        Ok(vec![])
                    }
                    trust_dns_resolver::error::ResolveErrorKind::Message(msg) => Err(anyhow!(msg)),
                    _ => Err(anyhow!("failed to resolve '{domain_str}': {e}")),
                },
                |v| Ok(v.iter().map(|r| r.to_string()).collect::<Vec<_>>()),
            )
            .map(|mut v| {
                if v.is_empty() {
                    None
                } else {
                    v.sort();
                    Some(v)
                }
            })?;

        let mx = dns_client
            .mx_lookup(domain_str)
            .await
            .map_or_else(
                |e| match *e.kind() {
                    trust_dns_resolver::error::ResolveErrorKind::NoRecordsFound { .. } => {
                        Ok(vec![])
                    }
                    trust_dns_resolver::error::ResolveErrorKind::Message(msg) => Err(anyhow!(msg)),
                    _ => Err(anyhow!("failed to resolve '{domain_str}': {e}")),
                },
                |v| {
                    Ok(v.iter()
                        .map(|r| MxRecord {
                            host: r.exchange().to_string(),
                            priority: r.preference(),
                        })
                        .collect::<Vec<_>>())
                },
            )
            .map(|mut v| match v.len() {
                0 => None,
                _ => {
                    v.sort_by(|a, b| a.priority.cmp(&b.priority));
                    Some(v)
                }
            })?;

        let (txt, spf) = dns_client.txt_lookup(domain_str).await.map_or_else(
            |e| match *e.kind() {
                trust_dns_resolver::error::ResolveErrorKind::NoRecordsFound { .. } => {
                    Ok((None, None))
                }
                trust_dns_resolver::error::ResolveErrorKind::Message(msg) => Err(anyhow!(msg)),
                _ => Err(anyhow!("failed to resolve '{domain_str}': {e}")),
            },
            |v| {
                let txt = v.iter().map(|r| r.to_string()).collect::<Vec<_>>();
                let spf_pos = txt.iter().position(|r| r.starts_with("v=spf1"));
                let spf = spf_pos.map(|pos| txt[pos].to_string());
                let txt = txt
                    .into_iter()
                    .enumerate()
                    .filter(|(idx, _)| spf_pos.is_none() || *idx != spf_pos.unwrap())
                    .map(|(_, r)| r)
                    .collect::<Vec<_>>();
                Ok((if txt.is_empty() { None } else { Some(txt) }, spf))
            },
        )?;

        let soa = dns_client.soa_lookup(domain_str).await.map_or_else(
            |e| match *e.kind() {
                trust_dns_resolver::error::ResolveErrorKind::NoRecordsFound { .. } => Ok(None),
                trust_dns_resolver::error::ResolveErrorKind::Message(msg) => Err(anyhow!(msg)),
                _ => Err(anyhow!("failed to resolve '{domain_str}': {e}")),
            },
            |v| {
                Ok(Some(
                    v.iter()
                        .map(|r| SoaRecord {
                            mname: r.mname().to_string(),
                            rname: r.rname().to_string(),
                            serial: r.serial(),
                            refresh: r.refresh() as u32,
                            retry: r.retry() as u32,
                            expire: r.expire() as u32,
                            minimum: r.minimum(),
                        })
                        .collect::<Vec<_>>(),
                ))
            },
        )?;

        Ok(Self {
            domain,
            ns,
            mx,
            spf,
            txt,
            soa,
        })
    }
}
