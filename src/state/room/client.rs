use std::net::SocketAddr;

use tokio_kcp::KcpStream;

use crate::engine::network::DEFAULT_KCP_CONFIG;
use crate::engine::network::peer::Peer;
use crate::state::room::{MessageHandler, ReceiverType};

pub struct Client {
    pub target: Peer,
    pub receiver: ReceiverType,
}

async fn get_target_receiver(addr: SocketAddr) -> anyhow::Result<(Peer, ReceiverType)> {
    let stream = KcpStream::connect(&DEFAULT_KCP_CONFIG, addr).await?;
    log::info!("Connected to {}", addr);

    let (receiver, handler) = MessageHandler::create();
    let target = Peer::new(stream, addr, handler);
    Ok((
        target,
        receiver,
    ))
}

impl Client {
    /// Should be called in `tokio` context
    pub async fn new(addr: SocketAddr) -> anyhow::Result<Self> {
        let (target, receiver) = get_target_receiver(addr).await?;
        Ok(Self {
            target,
            receiver,
        })
    }
}