mod gain;
mod runner;

use crate::vban;
use std::io::Result;
use std::net::{ToSocketAddrs, UdpSocket};

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
                gain::auto_gain_i16(v.vban_data, gain, &mut gain_acc);
            } else {
                println!("Invalid format type {:?}", v.vban_header.data_type());
            }
        }
        for t in tx_addrs {
            socket.send_to(&buf[..n], t)?;
        }
    }
}
