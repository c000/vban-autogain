use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Read;
use std::io::Result;

#[derive(Debug)]
pub struct VbanHeader {
    pub format_sr: u8,
    pub format_nbs: u8,
    pub format_nbc: u8,
    pub format_bit: u8,
    pub streamname: [u8; 16],
    pub nu_frame: u32,
}

#[derive(PartialEq, Debug)]
pub enum Protocol {
    Audio,
    Serial,
    Txt,
    Service,
    Undefined(u8),
}

#[derive(PartialEq, Debug)]
pub enum DataType {
    U8,
    I16,
    I24,
    I32,
    F32,
    F64,
    I12,
    I10,
}

#[derive(PartialEq, Debug)]
pub enum Codec {
    PCM,
    VBCA,
    VBCV,
    Undefined(u8),
    User,
}

impl VbanHeader {
    fn from_slice<R>(mut s: R) -> Result<VbanHeader>
    where
        R: Read,
    {
        let sr = s.read_u8()?;
        let nbs = s.read_u8()?;
        let nbc = s.read_u8()?;
        let bit = s.read_u8()?;
        let mut sn = [0; 16];
        s.read_exact(&mut sn)?;
        let nu = s.read_u32::<LittleEndian>()?;
        Ok(VbanHeader {
            format_sr: sr,
            format_nbs: nbs,
            format_nbc: nbc,
            format_bit: bit,
            streamname: sn,
            nu_frame: nu,
        })
    }

    pub fn protocol(&self) -> Protocol {
        match self.format_sr & 0xe0 {
            0x00 => Protocol::Audio,
            0x20 => Protocol::Serial,
            0x40 => Protocol::Txt,
            0x60 => Protocol::Service,
            x => Protocol::Undefined(x),
        }
    }

    pub fn data_type(&self) -> DataType {
        match self.format_bit & 0x07 {
            0 => DataType::U8,
            1 => DataType::I16,
            2 => DataType::I24,
            3 => DataType::I32,
            4 => DataType::F32,
            5 => DataType::F64,
            6 => DataType::I12,
            7 => DataType::I10,
            _ => panic!("unreachable pattern!"),
        }
    }

    pub fn codec(&self) -> Codec {
        match self.format_bit & 0xf0 {
            0x00 => Codec::PCM,
            0x10 => Codec::VBCA,
            0x20 => Codec::VBCV,
            0xf0 => Codec::User,
            x => Codec::Undefined(x),
        }
    }
}

#[derive(Debug)]
pub struct VbanPacket<'a> {
    pub vban_header: VbanHeader,
    pub vban_data: &'a mut [u8],
}

impl<'a> VbanPacket<'a> {
    pub fn from_mut_slice(s: &'a mut [u8]) -> Result<VbanPacket<'a>> {
        let (h, body) = s.split_at_mut(28);
        let header = VbanHeader::from_slice(&h[4..])?;
        Ok(VbanPacket {
            vban_header: header,
            vban_data: body,
        })
    }
}
