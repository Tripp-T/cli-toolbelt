use crate::{models::CidrNetwork, *};

#[derive(Parser, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Opts {
    #[clap(required = true)]
    addresses: Vec<CidrNetwork>,
}

pub async fn main(opts: &Opts) -> Result<()> {
    for (idx, network) in opts.addresses.iter().enumerate() {
        if idx != 0 {
            // separate each CIDR output with a newline
            println!();
        }
        if network.ip.is_ipv6() {
            // TODO: improve accuracy of IPv6 CIDR ranges to remove this warning
            warn!(target: "cidr", "[WARNING] IPv6 CIDR range total usable IPs liable to be incorrect.")
        }
        println!("CIDR:             {}/{}", network.ip, network.mask);
        println!("Starting IP:      {}", network.starting_ip);
        println!("Ending IP:        {}", network.ending_ip);
        println!("Usable IPs:       {}", network.total_addresses);
    }
    Ok(())
}
