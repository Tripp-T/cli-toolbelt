use crate::*;

#[derive(clap::Parser, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Opts {
    /// remote host to connect to
    #[clap(required = true)]
    host: Host,
    /// Port of remote host to connect to
    #[clap(required = true)]
    port: Port,
    /// Send input from stdin to the peer
    #[clap(short, long, default_value_t = true)]
    interactive: bool,
}

#[allow(unreachable_code)]
pub async fn main(opts: &mut Opts) -> anyhow::Result<()> {
    let addr = {
        let host = opts.host.get_ip().await?;
        let port = opts.port.0;
        SocketAddr::from_str(&format!("{host}:{port}"))
            .map_err(|e| anyhow!("invalid socket addr '{host}:{port}': {e}"))
    }?;
    info!(target: "client", "Connecting to {addr}");
    let stream = tokio::net::TcpStream::connect(addr).await?;
    info!(target: "client", "Connected to {addr}");
    let (mut reader, mut writer) = stream.into_split();

    if opts.interactive {
        tokio::spawn(async move {
            let mut buf = [0; 1024];
            loop {
                let n: usize = tokio::io::stdin().read(&mut buf).await?;
                if n == 0 {
                    info!(target: "client", "Received EOF from stdin, closing connection to peer");
                    exit(0);
                }
                writer.write_all(&buf[0..n]).await?;
            }
            anyhow::Ok(())
        });
    }

    let mut buf = [0; 1024];
    loop {
        let n = reader.read(&mut buf).await?;
        if n == 0 {
            info!(target: "client", "Connection closed by peer");
            exit(0);
        }
        tokio::io::stdout().write_all(&buf[0..n]).await?;
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Port(u16);

impl std::str::FromStr for Port {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<u16>() {
            Ok(port) => Ok(Self(port)),
            Err(e) => bail!("Invalid remote port '{s}': {e}"),
        }
    }
}
