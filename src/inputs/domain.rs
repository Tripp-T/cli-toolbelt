use std::{net::IpAddr, sync::atomic::AtomicBool};

use anyhow::{anyhow, bail};
use tokio::sync::{Mutex, RwLock};
use trust_dns_resolver::TokioAsyncResolver;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Domain(String);

impl std::fmt::Display for Domain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for Domain {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s.to_string())
    }
}

impl Domain {
    pub fn new(domain: String) -> anyhow::Result<Self> {
        if domain.parse::<IpAddr>().is_ok() {
            bail!("domain cannot be an IP address");
        }
        Self::is_rfc_compliant(&domain).map(|_| Self(format!("{}.", domain.to_lowercase())))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
    pub fn tld(&self) -> Option<&str> {
        self.0.split('.').rev().nth(1)
    }
    pub fn root_domain(&self) -> String {
        let mut parts = self.0.split('.');
        let tld = parts.next_back().unwrap();
        let domain = parts.next_back().unwrap();
        format!("{domain}.{tld}")
    }
    pub fn is_rfc_compliant(domain: &str) -> anyhow::Result<()> {
        let mut parts = domain.split('.');
        let tld = parts.next_back().unwrap();
        if tld.len() < 2 || tld.len() > 63 {
            bail!("TLD must be between 2 and 63 characters");
        }
        for part in parts {
            if part.is_empty() || part.len() > 63 {
                bail!("domain part must be between 1 and 63 characters");
            }
        }
        Ok(())
    }
    pub async fn get_ip(&self) -> anyhow::Result<IpAddr> {
        let domain = &self.0;
        TokioAsyncResolver::tokio_from_system_conf()
            .map_err(|e| anyhow!("failed to create DNS resolver: {e}"))?
            .lookup_ip(domain)
            .await
            .map_err(|e| {
                anyhow!(
                    "failed to resolve '{domain}': {error}",
                    error = match e.kind() {
                        trust_dns_resolver::error::ResolveErrorKind::NoRecordsFound { .. } =>
                            "no records found",
                        trust_dns_resolver::error::ResolveErrorKind::Message(msg) => msg,
                        _ => "unknown error",
                    }
                )
            })?
            .iter()
            .next()
            .ok_or(anyhow!("failed to resolve valid IP for '{domain}'"))
    }
}
