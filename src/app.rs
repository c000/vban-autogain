use crate::vban;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::Result;
use std::net::{ToSocketAddrs, UdpSocket};

fn i16_to_f32(mut s: &[u8]) -> Vec<f32> {
    let mut v = Vec::with_capacity(s.len() / 2);
    while let Ok(i) = s.read_i16::<LittleEndian>() {
        let a = i as f32;
        v.push(a);
    }
    v
}

fn auto_gain_i16(mut s: &mut [u8], gain: f32, gain_acc: &mut f32) {
    let sample_num = s.len() / 4;
    let gain_mul = gain.powf(sample_num as f32);
    *gain_acc *= gain_mul;

    let samples = i16_to_f32(s);
    let max_sample = samples
        .iter()
        .map(|v| v.abs())
        .fold(0.0 / 0.0, |x, y| y.max(x));
    let max_gain = i16::max_value() as f32 / max_sample;
    *gain_acc = gain_acc.min(max_gain);

    samples
        .iter()
        .map(|v| v * *gain_acc)
        .for_each(|v| s.write_i16::<LittleEndian>(v.floor() as i16).unwrap());
}

pub fn main<T>(rx_addr: T, tx_addrs: &[T], gain: f32) -> Result<()>
where
    T: ToSocketAddrs,
{
    let socket = UdpSocket::bind(rx_addr)?;
    let mut buf = [0; 1500];
    let mut gain_acc: f32 = 1.0;
    loop {
        let n = socket.recv(&mut buf)?;
        {
            let v = vban::VbanPacket::from_mut_slice(&mut buf[..n])?;
            if v.vban_header.data_type() == vban::DataType::I16 {
                auto_gain_i16(v.vban_data, gain, &mut gain_acc);
            } else {
                println!("Invalid format type {:?}", v.vban_header.data_type());
            }
        }
        for t in tx_addrs {
            socket.send_to(&buf[..n], t)?;
        }
    }
}
