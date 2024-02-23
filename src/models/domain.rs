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

static VALID_TLDS_URL: &str = "https://data.iana.org/TLD/tlds-alpha-by-domain.txt";
const CACHED_TLDS: &str = include_str!("../../dynamic_assets/tlds-alpha-by-domain.txt");

pub struct DomainValidator {
    valid_tlds: RwLock<Vec<String>>,
    refreshing: AtomicBool,
    has_refreshed: AtomicBool,
    refresh_finish_listeners: Mutex<Vec<tokio::sync::oneshot::Sender<()>>>,
}

impl DomainValidator {
    pub fn new() -> anyhow::Result<Self> {
        let valid_tlds = CACHED_TLDS
            .lines()
            .filter(|line| !line.starts_with('#'))
            .map(|line| line.to_lowercase())
            .collect::<Vec<_>>();
        Ok(Self {
            valid_tlds: RwLock::new(valid_tlds),
            refreshing: AtomicBool::new(false),
            has_refreshed: AtomicBool::new(false),
            refresh_finish_listeners: Mutex::new(Vec::new()),
        })
    }

    pub async fn is_valid_tld(&self, tld: &str) -> bool {
        tld == "local" || self.valid_tlds.read().await.contains(&tld.to_string())
    }

    pub fn is_refreshing(&self) -> bool {
        self.refreshing.load(std::sync::atomic::Ordering::Relaxed)
    }
    pub fn has_refreshed(&self) -> bool {
        self.has_refreshed
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// If `Ok(None)` is returned, the refresh is already in progress.
    /// If `Ok(Some(()))` is returned, the refresh has completed.
    /// If `Err(_)` is returned, the refresh failed.
    pub async fn refresh_valid_tlds(&self) -> anyhow::Result<Option<()>> {
        if self
            .refreshing
            .swap(true, std::sync::atomic::Ordering::Relaxed)
        {
            return Ok(None);
        }
        println!("Refreshing valid TLDs...");
        let new_tlds = reqwest::get(VALID_TLDS_URL)
            .await
            .map_err(|e| anyhow!("failed to fetch valid TLDs: {e}"))?
            .text()
            .await
            .map_err(|e| anyhow!("failed to parse valid TLDs: {e}"))?
            .lines()
            .filter(|line| !line.starts_with('#'))
            .map(|line| line.to_lowercase())
            .collect::<Vec<_>>();

        *self.valid_tlds.write().await = new_tlds;

        self.has_refreshed
            .store(true, std::sync::atomic::Ordering::Relaxed);
        self.refreshing
            .store(false, std::sync::atomic::Ordering::Relaxed);

        let mut listeners = self.refresh_finish_listeners.lock().await;
        for listener in listeners.drain(..) {
            listener.send(()).unwrap();
        }

        Ok(Some(()))
    }

    pub async fn validate(&self, domain: &Domain) -> anyhow::Result<()> {
        let tld = domain
            .tld()
            .ok_or_else(|| anyhow!("failed to validate domain '{domain}': domain has no TLD?"))?;
        if !self.is_valid_tld(tld).await {
            if self.has_refreshed() {
                bail!("failed to validate domain '{domain}': TLD '{tld}' is not valid");
            }
            match self.refresh_valid_tlds().await? {
                // refresh was successful
                Some(_) => {}
                // refresh is already in progress
                None => {
                    let (tx, rx) = tokio::sync::oneshot::channel();
                    self.refresh_finish_listeners.lock().await.push(tx);
                    rx.await?;
                }
            }
            if !self.is_valid_tld(tld).await {
                bail!("failed to validate domain '{domain}': TLD '{tld}' is not valid");
            }
        }
        Ok(())
    }
}
