use std::{
    error, io,
    net::{SocketAddr, ToSocketAddrs},
    result,
    sync::Arc,
    vec::Vec,
};

use tokio::{io::AsyncBufReadExt, net::UdpSocket, sync::RwLock, task};

use super::gain;
use crate::vban;

pub struct Runner {
    rx_addr: SocketAddr,
    tx_addrs: Arc<RwLock<Vec<SocketAddr>>>,
    gain_per_sample: f32,

    gain: Arc<RwLock<f32>>,
}

impl Runner {
    pub fn new<T>(a: T, g: f32) -> io::Result<Runner>
    where
        T: ToSocketAddrs,
    {
        let rx_addr = a.to_socket_addrs()?.next().ok_or(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "rx address parse error",
        ))?;
        Ok(Runner {
            rx_addr: rx_addr,
            tx_addrs: Arc::new(RwLock::new(Vec::new())),
            gain_per_sample: g,

            gain: Arc::new(RwLock::new(1.0)),
        })
    }

    pub async fn add_tx_addrs<T>(&self, a: T) -> result::Result<(), Box<dyn error::Error>>
    where
        T: ToSocketAddrs,
    {
        let tx_addr = a.to_socket_addrs()?.next().ok_or(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "tx address parse error",
        ))?;

        let mut tx_vec = self.tx_addrs.write().await;
        (*tx_vec).push(tx_addr);

        Ok(())
    }

    pub async fn spawn_pipe_loop(&self) -> io::Result<task::JoinHandle<()>> {
        let socket = UdpSocket::bind(self.rx_addr).await?;
        let tx_addrs = self.tx_addrs.clone();
        let mut buf = [0; 1500];

        let gain_per_sample = self.gain_per_sample;
        let gain = self.gain.clone();

        Ok(task::spawn(async move {
            loop {
                let n = socket.recv(&mut buf).await.unwrap();
                {
                    let v = vban::VbanPacket::from_mut_slice(&mut buf[..n]).unwrap();
                    if v.vban_header.data_type() == vban::DataType::I16 {
                        let mut g = gain.write().await;
                        gain::auto_gain_i16(v.vban_data, gain_per_sample, &mut *g);
                    } else {
                        println!("Invalid format type {:?}", v.vban_header.data_type());
                    }
                }
                for t in tx_addrs.read().await.iter() {
                    socket.send_to(&buf[..n], t).await.unwrap();
                }
            }
        }))
    }

    pub async fn repl(&self) -> io::Result<()> {
        let mut lines = tokio::io::BufReader::new(tokio::io::stdin()).lines();

        while let Some(line) = lines.next_line().await? {
            if line == "exit" {
                break;
            }
        }
        Ok(())
    }
}
