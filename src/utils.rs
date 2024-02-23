use std::net::IpAddr;

pub fn validate_fqdn_or_ip(s: &str) -> anyhow::Result<()> {
    if s.parse::<IpAddr>().is_ok() {
        return Ok(());
    } else if s.is_empty() {
        return Err(anyhow::anyhow!("FQDN cannot be empty"));
    } else if s.len() > 255 {
        return Err(anyhow::anyhow!("FQDN cannot be longer than 255 characters"));
    } else if s.ends_with('.') {
        return Err(anyhow::anyhow!("FQDN cannot end with a period"));
    }
    for label in s.split('.') {
        if label.is_empty() {
            return Err(anyhow::anyhow!("FQDN cannot contain empty labels"));
        }
        if label.len() > 63 {
            return Err(anyhow::anyhow!(
                "FQDN labels cannot be longer than 63 characters"
            ));
        }
        if label.starts_with('-') || label.ends_with('-') {
            return Err(anyhow::anyhow!(
                "FQDN labels cannot start or end with a hyphen"
            ));
        }
        if label
            .chars()
            .any(|c| !c.is_ascii_alphanumeric() && c != '-')
        {
            return Err(anyhow::anyhow!(
                "FQDN labels can only contain alphanumeric characters and hyphens"
            ));
        }
    }
    Ok(())
}

pub fn port_from_protocol(proto: &str) -> u16 {
    match proto.to_lowercase().as_str() {
        "http" => 80,
        "https" => 443,
        "ftp" => 21,
        "ssh" => 22,
        "smtp" => 25,
        "pop3" => 110,
        "imap" => 143,
        _ => 0,
    }
}
