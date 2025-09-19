use std::{error::Error, time::Duration, u64};


use libp2p::{noise, ping, swarm::SwarmEvent, tcp, yamux, Multiaddr, Swarm};
use tokio::sync::mpsc;
use tracing_subscriber::EnvFilter;

pub struct P2pNode {
    pub local_addr: Option<Multiaddr>,
}

pub async fn init_node(
    tx: mpsc::Sender<Multiaddr>,
) -> Result<Swarm<ping::Behaviour>, Box<dyn Error>> {
    let mut swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|_| ping::Behaviour::default())?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX)))
        .build();

    swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

    Ok(swarm)
}
pub fn dial(swarm: &mut Swarm<ping::Behaviour>, addr: &str) -> Result<(), Box<dyn Error>> {
        let remote: Multiaddr = addr.parse()?;
        swarm.dial(remote)?;
        Ok(())
    }
