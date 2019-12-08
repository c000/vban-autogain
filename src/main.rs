extern crate vban_autogain;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use clap::{App, Arg};
use std::io::Result;
use std::net::{ToSocketAddrs, UdpSocket};
use std::str::FromStr;
use vban_autogain::vban::VbanPacket;

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

fn amain(rx_addr: &str, tx_addr: &str, gain: f32) -> Result<()> {
    let socket = UdpSocket::bind(rx_addr)?;
    let mut buf = [0; 1500];
    let mut gain_acc: f32 = 1.0;
    loop {
        let n = socket.recv(&mut buf)?;
        {
            let v = VbanPacket::from_mut_slice(&mut buf[..n])?;
            if v.vban_header.data_type() == vban_autogain::vban::DataType::I16 {
                auto_gain_i16(v.vban_data, gain, &mut gain_acc);
            } else {
                println!("Invalid format type {:?}", v.vban_header.data_type());
            }
        }
        socket.send_to(&buf[..n], tx_addr)?;
    }
}

fn main() {
    let socket_addrs_validator = |s: String| match s.to_socket_addrs() {
        Ok(_) => core::result::Result::Ok(()),
        Err(s) => core::result::Result::Err(s.to_string()),
    };

    let matches = App::new("VBAN Autogain")
        .author("c000")
        .arg(
            Arg::with_name("listen")
                .short("l")
                .long("listen")
                .help("Listen address")
                .default_value("0.0.0.0:6900")
                .takes_value(true)
                .validator(socket_addrs_validator),
        )
        .arg(
            Arg::with_name("remote")
                .short("r")
                .long("remote")
                .help("Remote address")
                .required(true)
                .takes_value(true)
                .validator(socket_addrs_validator),
        )
        .arg(
            Arg::with_name("gain")
                .short("g")
                .long("gain")
                .help("Gain per sample")
                .default_value("1e-5")
                .takes_value(true)
                .validator(|s| {
                    f32::from_str(s.as_str())
                        .map(|_| ())
                        .map_err(|e| e.to_string())
                }),
        )
        .get_matches();

    let rx_addr = matches.value_of("listen").unwrap();
    let tx_addr = matches.value_of("remote").unwrap();
    let gain_db = f32::from_str(matches.value_of("gain").unwrap()).unwrap();
    let gain = (10.0 as f32).powf(gain_db / 20.0);

    amain(rx_addr, tx_addr, gain).unwrap();
}
