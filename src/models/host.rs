use std::{
    fmt::{Display, Formatter},
    net::IpAddr,
};

use anyhow::anyhow;
use trust_dns_resolver::TokioAsyncResolver;

use crate::utils;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Host {
    pub input: String,
    pub ip: Option<IpAddr>,
}

impl std::str::FromStr for Host {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        utils::validate_fqdn_or_ip(s).map(|_| Self {
            input: s.to_owned(),
            ip: None,
        })
    }
}

impl Display for Host {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.input)
    }
}

impl Host {
    pub async fn get_ip(&mut self) -> anyhow::Result<IpAddr> {
        if let Some(ip) = &self.ip {
            // return IP lookup result if already resolved
            return Ok(*ip);
        }
        let v = &self.input;
        if let Ok(ip) = v.parse::<IpAddr>() {
            // return IP if input already is one
            self.ip = Some(ip);
            return Ok(ip);
        };
        // resolve IP from DNS
        TokioAsyncResolver::tokio_from_system_conf()
            .map_err(|e| anyhow!("failed to create DNS resolver: {e}"))?
            .lookup_ip(v)
            .await
            .map_err(|e| {
                anyhow!(
                    "failed to resolve '{v}': {}",
                    match e.kind() {
                        trust_dns_resolver::error::ResolveErrorKind::NoRecordsFound { .. } =>
                            "no records found",
                        trust_dns_resolver::error::ResolveErrorKind::Message(msg) => msg,
                        _ => "unknown error",
                    }
                )
            })?
            .iter()
            .next()
            .ok_or(anyhow!("failed to resolve valid IP for '{v}'"))
            .map(|ip| {
                self.ip = Some(ip);
                ip
            })
    }
}
