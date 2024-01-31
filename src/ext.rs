use anyhow::anyhow;

pub const PACKET_HEADER: &'static [u8] = b"rPtm";
pub const CURRENT_VERSION: u32 = 0;

pub trait PacketWriteExt {
    fn add_header(&mut self);

    fn write_string(&mut self, str: &str);
}

impl PacketWriteExt for Vec<u8> {
    fn add_header(&mut self) {
        self.extend_from_slice(PACKET_HEADER);
        self.extend_from_slice(&CURRENT_VERSION.to_be_bytes());
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

            use bytes::Buf;
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