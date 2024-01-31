use std::future::Future;
use std::net::SocketAddr;
use std::time::Duration;

use tokio_kcp::{KcpConfig, KcpNoDelayConfig};

use crate::network::peer::Peer;

pub mod server;
pub mod peer;

#[allow(unused)]
/// The handler to handle the message from `Peer`
pub trait DataHandler: Send + 'static {
    /// `src` The address that sent the data
    /// `data` The data received from `src`
    /// Return true means successful
    fn handle<'a>(&'a mut self, src: &'a Peer, data: &'a [u8]) -> Box<dyn Future<Output=bool> + Unpin + Send + 'a>;
}

pub trait DataHandlerGenerator: Send + 'static {
    fn generate(&self, addr: SocketAddr) -> Box<dyn DataHandler>;
}


#[allow(unused)]
#[derive(Debug)]
pub enum NetworkMessage {
    Rely(Vec<u8>),
    Once(Vec<u8>),
}

#[allow(unused)]
pub const DEFAULT_KCP_CONFIG: KcpConfig = KcpConfig {
    mtu: 1400,
    nodelay: KcpNoDelayConfig {
        nodelay: true,
        interval: 10,
        resend: 2,
        nc: true,
    },
    wnd_size: (256, 256),
    session_expire: Duration::from_secs(60),
    flush_write: false,
    flush_acks_input: false,
    stream: false,
};
