//! The room states
//! Network packet types for now
//! * `join` if sent from client, means connected and following utf8 string as user name , else following usize(be) means your user id.
//! * S2C `room` `user` `join` <id: usize> <name: string>
//! * S2C `room` `user` `left` <id: usize>
//! * S2C `room` `chat` <id: usize> <string>
//! * S2C `voip` <id: usize> <bytes>
//!   <hr>
//! * C2S `chat` <msg: string>
//! * C2S `voip` <bytes: VoiceData>

use std::net::SocketAddr;

use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

pub use join::*;

use crate::engine::network::DataHandler;
use crate::engine::network::peer::Peer;

pub mod join;
pub mod client;
mod connecting;
mod bank;

#[derive(Clone)]
pub struct MessageHandler {
    pub sender: UnboundedSender<(SocketAddr, Vec<u8>)>,
}

impl DataHandler for MessageHandler {
    fn handle(&self, src: &Peer, data: &[u8]) -> bool {
        let x = self.sender.send((src.addr, Vec::from(data))).is_ok();
        x
    }
}

pub type ReceiverType = UnboundedReceiver<(SocketAddr, Vec<u8>)>;


impl MessageHandler {
    pub fn new(sender: UnboundedSender<(SocketAddr, Vec<u8>)>) -> Self {
        Self { sender }
    }
    pub fn create() -> (ReceiverType, Self) {
        let (s, r) = unbounded_channel();
        (r, MessageHandler::new(s))
    }
}
