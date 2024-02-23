use std::{fs, io, path::Path};

use anyhow::{anyhow, bail, Result};

const DYNAMIC_ASSETS_DIR: &str = "./dynamic_assets";

static TLD_LIST_URL: &str = "https://data.iana.org/TLD/tlds-alpha-by-domain.txt";
static TLD_LIST_FILE: &str = "tlds-alpha-by-domain.txt";

pub fn main() -> Result<()> {
    let assets_dir = Path::new(DYNAMIC_ASSETS_DIR);
    if !assets_dir.exists() {
        fs::create_dir(DYNAMIC_ASSETS_DIR)?;
    }

    // Download the TLD list if it doesn't exist or is older than 1 day

    let tld_file = assets_dir.join(TLD_LIST_FILE);
    let tld_metadata = match tld_file.metadata() {
        Ok(metadata) => Some(metadata),
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => None,
            _ => bail!(
                "failed to get metadata for TLD list file '{tld_file_path}': {e}",
                tld_file_path = tld_file.display()
            ),
        },
    };
    if tld_metadata.is_none()
        || tld_metadata.unwrap().modified()?
            < (chrono::Utc::now() - chrono::Duration::days(1)).into()
    {
        let tld_list = reqwest::blocking::get(TLD_LIST_URL)
            .map_err(|e| anyhow!("failed to fetch TLD list: {e}"))?
            .text()
            .map_err(|e| anyhow!("failed to parse TLD list: {e}"))?
            .lines()
            .filter(|line| !line.starts_with('#'))
            .map(|line| line.to_lowercase())
            .collect::<Vec<_>>();
        fs::write(tld_file, tld_list.join("\n"))
            .map_err(|e| anyhow!("failed to save TLD list: {e}"))?;
    }

    Ok(())
}
