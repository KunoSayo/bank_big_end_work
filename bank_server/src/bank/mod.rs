//! Currently packet from master stream -> connection -> bank data handler
//!
//! for all string: u16 len and utf coded str
//!
//! Packet format: `<header> <version: u32> <content>`
//!
//! Packet header: rPtm
//!
//! version: 0
//!
//! Contents:
//!
//! Server to client packets:
//! * Return main menu with info (b"menu") (id: u32) (name: String) (balance: u32) (phone_number: String)
//! * Normal tip and do nothing (b"msgb") (msg: String)
//! * Error (and disconnect) (b"errr") (reason: String)
//! *
//!

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::future::Future;

use bytes::Buf;
use log::trace;

use crate::bank::ext::PacketWriteExt;
use crate::bank::handlers::BankDataHandler;
use crate::bank::server::BankServer;
use crate::network::{DataHandler, NetworkMessage};
use crate::network::peer::Peer;

mod handlers;
pub mod server;
pub mod user;
pub mod ext;

pub const PACKET_HEADER: &'static [u8] = b"rPtm";
pub const CURRENT_VERSION: u32 = 0;

pub struct BankConnection {
    bank_server: BankServer,
    handler: Box<dyn BankDataHandler>,
}

impl BankConnection {
    pub fn new(bank_server: BankServer) -> Self {
        Self { bank_server, handler: Box::new(handlers::HandleLogin::default()) }
    }
}

impl DataHandler for BankConnection {
    fn handle<'a>(&'a mut self, src: &'a Peer, data: &'a [u8]) -> Box<dyn Future<Output=bool> + Send + Unpin + 'a> {
        trace!("Handle connection packet for len: {}", data.len());
        if data.len() < 8 || &data[0..4] != b"rPtm" {
            return Box::new(Box::pin(async { false }));
        }
        let version = (&data[4..8]).get_u32();

        if version != CURRENT_VERSION {
            return Box::new(Box::pin(async { false }));
        }


        let task = async {
            let handler_task = self.handler.handle(&self.bank_server, src, &data[8..]);
            let result = handler_task.await;
            match result {
                Ok(x) => {
                    if let Some(x) = x {
                        self.handler = x;
                    }
                    true
                }
                Err(e) if e.is::<UserInputError>() => {
                    let mut data = Vec::<u8>::new();
                    data.add_header();
                    data.extend_from_slice(b"msgb");
                    data.write_string(&format!("{:?}", e.downcast::<UserInputError>().unwrap().msg));
                    let _ = src.sender.send(NetworkMessage::Rely(data));
                    true
                }
                Err(e) => {
                    log::error!("Handler handled packet error for {:?}", e);
                    let mut data = Vec::<u8>::new();
                    data.add_header();
                    data.extend_from_slice(b"errr");
                    data.write_string(&format!("{:?}", e));
                    let _ = src.sender.send(NetworkMessage::Rely(data));
                    false
                }
            }
        };
        Box::new(Box::pin(task))
    }
}


fn add_fixed_header(vec: &mut Vec<u8>) {
    vec.extend_from_slice(PACKET_HEADER);
    vec.extend_from_slice(&CURRENT_VERSION.to_be_bytes());
}


#[derive(Debug)]
pub struct UserInputError {
    pub msg: &'static str,
}

impl UserInputError {
    pub fn new(msg: &'static str) -> Self {
        Self { msg }
    }
}

impl Display for UserInputError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.msg)
    }
}

impl Error for UserInputError {}
