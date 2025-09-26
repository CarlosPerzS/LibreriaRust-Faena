use std::{error::Error, time::Duration, u64};

use futures::StreamExt;
use libp2p::{
    Multiaddr, Swarm, noise,
    ping::{self, Event},
    swarm::SwarmEvent,
    tcp, yamux,
};
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum P2PMessage {
    NewAddress(Multiaddr),
    PeerEvent(String),
}

pub async fn start_node(
    tx: mpsc::Sender<P2PMessage>,
    remote_addr: Option<String>,
) -> Result<(), Box<dyn Error>> {
    println!("Inicializando el nodo...");

    // Crear swarm
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

    let listen_addr: Multiaddr = "/ip4/0.0.0.0/tcp/0".parse()?;
    swarm.listen_on(listen_addr)?;

    // llamar
    if let Some(addr) = remote_addr {
        let remote: Multiaddr = addr.parse()?;
        swarm.dial(remote)?;
        println!("Llamando a {addr}");
    }

    let mut event_count = 0;
    const MAX_EVENTS: usize = 10; //Limite de llamadas

    while event_count < MAX_EVENTS {
        match swarm.select_next_some().await {
            SwarmEvent::NewListenAddr { address, .. } => {
                println!("Escuchando en {address}");
                let _ = tx.send(P2PMessage::NewAddress(address)).await;
                event_count += 1;
            }
            SwarmEvent::Behaviour(event) => {
                // ← SOLO UN BLOQUE
                // Crear mensaje personalizado basado en el evento
                let mensaje = match event {
                    ping::Event {
                        peer,
                        connection,
                        result,
                    } => match result {
                        Ok(duration) => {
                            format!(
                                " Ping exitoso a: {} - en solo: {:.1}ms",
                                peer.to_string().chars().take(5).collect::<String>() + "...",
                                duration.as_millis()
                            )
                        }
                        Err(failure) => {
                            format!(
                                "Ping fallido a {}: {}",
                                peer.to_string().chars().take(5).collect::<String>() + "...",
                                failure
                            )
                        }
                    },
                };

                let _ = tx.send(P2PMessage::PeerEvent(mensaje.clone())).await;
                println!("{}", mensaje);
                event_count += 1;
            }
            _ => {
                event_count += 1;
            }
        }
    }

    println!("Nodo P2P iniciado correctamente");
    Ok(())
}
