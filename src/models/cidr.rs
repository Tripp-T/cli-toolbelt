use anyhow::{anyhow, bail, Result};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CidrNetwork {
    /// The IP address provided on creation of the CidrNetwork
    pub ip: IpAddr,
    /// The mask of the CIDR
    pub mask: u8,
    /// The first IP address in the CIDR range
    pub starting_ip: IpAddr,
    /// The last IP address in the CIDR range
    pub ending_ip: IpAddr,
    /// The total number of usable IP addresses in the CIDR range
    pub total_addresses: u128,
}

impl std::str::FromStr for CidrNetwork {
    type Err = anyhow::Error;

    /// Parse a CIDR string into a CidrNetwork struct
    ///
    /// # Examples
    ///
    /// `192.168.0.0/24`
    ///
    /// `2001:db8::/32`
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split('/').collect::<Vec<_>>();
        if parts.len() != 2 {
            bail!("Invalid CIDR: '{s}'");
        }
        let ip = parts[0]
            .parse::<IpAddr>()
            .map_err(|e| anyhow!("Invalid IP address in CIDR '{s}': {e}"))?;
        let mask = parts[1]
            .parse::<u8>()
            .map_err(|e| anyhow!("Invalid mask in CIDR '{s}': {e}"))?;

        CidrNetwork::from_ip(ip, mask)
    }
}

impl std::fmt::Display for CidrNetwork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}/{}", self.ip, self.mask)
    }
}

impl CidrNetwork {
    pub fn from_ip(ip: IpAddr, input_mask: u8) -> Result<CidrNetwork> {
        match ip {
            IpAddr::V4(ip) => CidrNetwork::from_ipv4(ip, input_mask),
            IpAddr::V6(ip) => CidrNetwork::from_ipv6(ip, input_mask),
        }
    }

    pub fn from_ipv4(sample_ip: Ipv4Addr, input_mask: u8) -> Result<CidrNetwork> {
        let mask = 32 - input_mask;
        let mask = 2_u32.pow(mask as u32);

        let octets = sample_ip.octets();

        let starting_ip = (u32::from_be_bytes(octets) & !mask) + 1;
        let broadcast_ip = starting_ip + mask - 1;
        let ending_ip = broadcast_ip - 1;

        Ok(CidrNetwork {
            ip: sample_ip.into(),
            mask: input_mask,
            starting_ip: Ipv4Addr::from(starting_ip).into(),
            ending_ip: Ipv4Addr::from(ending_ip).into(),
            total_addresses: mask as u128,
        })
    }

    pub fn from_ipv6(sample_ip: Ipv6Addr, input_mask: u8) -> Result<CidrNetwork> {
        let mask = 128 - input_mask;
        let mask = 2_u128.pow(mask as u32) - 1;

        let starting_ip = (u128::from_be_bytes(sample_ip.octets()) & !mask) + 1;
        let broadcast_ip = starting_ip + mask;
        let ending_ip = broadcast_ip - 1;

        Ok(CidrNetwork {
            ip: sample_ip.into(),
            mask: input_mask,
            starting_ip: Ipv6Addr::from(starting_ip).into(),
            ending_ip: Ipv6Addr::from(ending_ip).into(),
            total_addresses: mask,
        })
    }
}

#[test]
fn test_cidr_ipv4_24() -> Result<()> {
    let cidr = CidrNetwork::from_ipv4(Ipv4Addr::new(192, 168, 0, 0), 24)?;
    assert_eq!(cidr.ip, IpAddr::V4(Ipv4Addr::new(192, 168, 0, 0)));
    assert_eq!(cidr.mask, 24);
    assert_eq!(cidr.starting_ip, IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1)));
    assert_eq!(cidr.ending_ip, IpAddr::V4(Ipv4Addr::new(192, 168, 0, 255)));
    assert_eq!(cidr.total_addresses, 256);
    Ok(())
}
#[test]
fn test_cidr_ipv4_16() -> Result<()> {
    let cidr = CidrNetwork::from_ipv4(Ipv4Addr::new(192, 168, 0, 0), 16)?;
    assert_eq!(cidr.ip, IpAddr::V4(Ipv4Addr::new(192, 168, 0, 0)));
    assert_eq!(cidr.mask, 16);
    assert_eq!(cidr.starting_ip, IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1)));
    assert_eq!(
        cidr.ending_ip,
        IpAddr::V4(Ipv4Addr::new(192, 168, 255, 255))
    );
    assert_eq!(cidr.total_addresses, 65536);
    Ok(())
}
#[test]
fn test_cidr_ipv4_8() -> Result<()> {
    let cidr = "10.0.0.0/8".parse::<CidrNetwork>().unwrap();
    assert_eq!(cidr.ip, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 0)));
    assert_eq!(cidr.mask, 8);
    assert_eq!(cidr.starting_ip, IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)));
    assert_eq!(cidr.ending_ip, IpAddr::V4(Ipv4Addr::new(10, 255, 255, 255)));
    assert_eq!(cidr.total_addresses, 16777216);
    Ok(())
}
#[test]
fn text_cidr_from_str() -> Result<()> {
    assert!(
        "192.168.10.0/24".parse::<CidrNetwork>().is_ok(),
        "Failed to parse IPv4 CIDR from str"
    );
    Ok(())
}
