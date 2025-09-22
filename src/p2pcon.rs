use std::{error::Error, time::Duration, u64};

use futures::StreamExt;
use libp2p::{Multiaddr, Swarm, noise, ping, swarm::SwarmEvent, tcp, yamux};
use tokio::sync::mpsc;

use std::sync::Arc;
use tokio::sync::Mutex;

pub async fn init_node(
    tx: mpsc::Sender<Multiaddr>,
) -> Result<Arc<Mutex<Swarm<ping::Behaviour>>>, Box<dyn Error>> {
    let swarm = libp2p::SwarmBuilder::with_new_identity()
        .with_tokio()
        .with_tcp(
            tcp::Config::default(),
            noise::Config::new,
            yamux::Config::default,
        )?
        .with_behaviour(|_| ping::Behaviour::default())?
        .with_swarm_config(|cfg| cfg.with_idle_connection_timeout(Duration::from_secs(u64::MAX)))
        .build();

    let swarm = Arc::new(Mutex::new(swarm));

    {
        let swarm = Arc::clone(&swarm);
        tokio::spawn(async move {
            loop {
                let mut swarm = swarm.lock().await;
                let event = swarm.select_next_some().await;
                if let SwarmEvent::NewListenAddr { address, .. } = event {
                    let _ = tx.send(address).await;
                }
            }
        });
    }

    Ok(swarm)
}

pub async fn start_node(
    tx: mpsc::Sender<Multiaddr>,
    remote_addr: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let swarm = init_node(tx).await?;

    if let Some(addr) = remote_addr {
        let remote: Multiaddr = addr.parse()?;
        let mut swarm = swarm.lock().await;
        swarm.dial(remote)?;
        println!("Llamando a {addr}");
    }

    loop {
        let mut swarm = swarm.lock().await;
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Escuchando en {address:?}");
            }
            SwarmEvent::Behaviour(event) => {
                println!("{event:?}");
            }
            _ => {}
        }
    }
}