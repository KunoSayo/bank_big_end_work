use anyhow::anyhow;
use bytes::Buf;
use crate::bank::add_fixed_header;

pub trait PacketWriteExt {
    fn add_header(&mut self);

    fn write_string(&mut self, str: &str);
}

impl PacketWriteExt for Vec<u8> {
    /// Add fixed header bytes and version
    fn add_header(&mut self) {
        add_fixed_header(self);
    }

    fn write_string(&mut self, str: &str) {
        let data = str.as_bytes();
        let len = data.len();
        self.extend_from_slice(&(len as u16).to_be_bytes());
        self.extend_from_slice(data);
    }
}


pub trait PacketReadExt {
    fn read_packet_string(&mut self) -> anyhow::Result<String>;
}

impl PacketReadExt for &[u8] {
    fn read_packet_string(&mut self) -> anyhow::Result<String> {
        if self.len() < 2 {
            Err(anyhow!("Not enough len to read string"))
        } else {
            let len = self.get_u16();
            if self.len() < len as usize {
                Err(anyhow!("Not enough len to read string"))?
            }
            let str = String::from_utf8(Vec::from(&self[..len as usize]))?;
            *self = &self[len as usize..];
            Ok(str)
        }
    }
}