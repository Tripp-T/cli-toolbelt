use std::{net::IpAddr, sync::atomic::AtomicU16};

use surge_ping::SurgeError;

pub struct Pinger {
    v4_client: surge_ping::Client,
    v6_client: surge_ping::Client,
    next_id: AtomicU16,
}

impl Pinger {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            v4_client: surge_ping::Client::new(
                &surge_ping::Config::builder()
                    .kind(surge_ping::ICMP::V4)
                    .build(),
            )
            .map_err(|e| anyhow::anyhow!("failed to create IPv4 ping client: {e}"))?,
            v6_client: surge_ping::Client::new(
                &surge_ping::Config::builder()
                    .kind(surge_ping::ICMP::V6)
                    .build(),
            )
            .map_err(|e| anyhow::anyhow!("failed to create IPv6 ping client: {e}"))?,
            next_id: AtomicU16::new(0),
        })
    }

    /// increments next ID and returns the previous unused value
    fn use_next_id(&self) -> surge_ping::PingIdentifier {
        surge_ping::PingIdentifier(
            self.next_id
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst),
        )
    }

    /// Send a single ping to the given IP address.
    /// Payload is a Vec<0u8> with length `payload_length`.
    /// If `payload_length` is `None`, a default payload length of 1000 bytes will be used.
    /// Returns the round-trip time of the ping.
    pub async fn one_off(
        &self,
        ip: IpAddr,
        payload_length: Option<usize>,
    ) -> anyhow::Result<std::time::Duration> {
        let client = match ip {
            IpAddr::V4(_) => &self.v4_client,
            IpAddr::V6(_) => &self.v6_client,
        };

        let mut payload = Vec::with_capacity(payload_length.unwrap_or(1000));
        payload.fill(0u8);

        client
            .pinger(ip, self.use_next_id())
            .await
            .ping(surge_ping::PingSequence(0), &payload)
            .await
            .map(|(_, d)| d)
            .map_err(|e| match e {
                SurgeError::Timeout { seq: _ } => anyhow::anyhow!("request timeout"),
                e => e.into(),
            })
    }
}
