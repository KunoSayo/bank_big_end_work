use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use log::info;
use tokio::{pin, select};
use tokio::net::ToSocketAddrs;
use tokio::sync::RwLock;
use tokio_kcp::KcpListener;


use crate::network::{DataHandlerGenerator, DEFAULT_KCP_CONFIG};
use crate::network::peer::Peer;

/// The server object which could be clone
#[allow(unused)]
#[derive(Clone, Debug)]
pub struct Server {
    pub running: Arc<AtomicBool>,
    /// The peers still running
    pub peers: Arc<RwLock<HashMap<SocketAddr, Peer>>>,
}

#[allow(unused)]
impl Server {
    /// Construct the server and start to listen messages.
    pub async fn new(listen_ip: impl ToSocketAddrs, handler: impl DataHandlerGenerator) -> anyhow::Result<Self> {
        let listener = KcpListener::bind(DEFAULT_KCP_CONFIG, listen_ip).await?;
        let this = Self {
            running: Arc::new(AtomicBool::new(true)),
            peers: Default::default(),
        };
        tokio::spawn(this.clone().run_loop(listener, handler));
        Ok(this)
    }

    /// Construct the server and start to listen messages.
    pub async fn run_block(listen_ip: impl ToSocketAddrs, handler: impl DataHandlerGenerator) -> anyhow::Result<()> {
        let listener = KcpListener::bind(DEFAULT_KCP_CONFIG, listen_ip).await?;
        let this = Self {
            running: Arc::new(AtomicBool::new(true)),
            peers: Default::default(),
        };
        this.run_loop(listener, handler).await;
        Ok(())
    }

    async fn run_loop(self, mut listener: KcpListener, handler: impl DataHandlerGenerator) {
        info!("Server looping");
        while self.running.load(Ordering::Acquire) {
            let sleep = tokio::time::sleep(Duration::from_secs(60));
            pin!(sleep);
            select! {
                packet = listener.accept() => {
                    match packet {
                        Ok((stream, addr)) => {
                            info!("Accepted KcpStream from {:?}", addr);
                            let peer = Peer::new(stream, addr, handler.generate(addr.clone()));
                            let mut write = self.peers.write().await;
                            if let Some(old_peer) = write.insert(peer.addr, peer) {
                                old_peer.listening.store(false, Ordering::Relaxed);
                            }

                            write.retain(|_, p| p.listening.load(Ordering::Relaxed));
                        }
                        Err(e) => {
                            log::warn!("accept packet from listener failed for {:?}", e);
                        }
                    }
                }
                _ = &mut sleep => {
                    let mut write = self.peers.write().await;
                    write.retain(|_, p| p.listening.load(Ordering::Relaxed));
                }
            }
        }
        info!("Server loop exited");
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}