#[macro_use]
extern crate clap;

use clap::{App, Arg};
use std::net::ToSocketAddrs;
use std::str::FromStr;
use vban_autogain::app;

fn main() {
    let socket_addrs_validator = |s: String| match s.to_socket_addrs() {
        Ok(_) => core::result::Result::Ok(()),
        Err(s) => core::result::Result::Err(s.to_string()),
    };

    let matches = App::new("VBAN Autogain")
        .author("c000")
        .version(crate_version!())
        .arg(
            Arg::with_name("listen")
                .short("l")
                .long("listen")
                .help("Listen address")
                .default_value("0.0.0.0:6980")
                .takes_value(true)
                .validator(socket_addrs_validator),
        )
        .arg(
            Arg::with_name("remote")
                .short("r")
                .long("remote")
                .help("Remote address")
                .multiple(true)
                .takes_value(true)
                .validator(socket_addrs_validator),
        )
        .arg(
            Arg::with_name("gain")
                .short("g")
                .long("gain")
                .help("Gain per sample")
                .default_value("1e-3")
                .takes_value(true)
                .validator(|s| {
                    f32::from_str(s.as_str())
                        .map(|_| ())
                        .map_err(|e| e.to_string())
                }),
        )
        .get_matches();

    let rx_addr = matches.value_of("listen").unwrap();
    let tx_addrs = matches
        .values_of("remote")
        .unwrap_or(clap::Values::default())
        .collect::<std::boxed::Box<[_]>>();
    let gain_db = f32::from_str(matches.value_of("gain").unwrap()).unwrap();
    let gain = 10.0_f32.powf(gain_db / 20.0_f32);

    app::main(rx_addr, tx_addrs.as_ref(), gain).unwrap();
}
