use std::{
    error,
    fmt::Write,
    io,
    net::{SocketAddr, ToSocketAddrs},
    result,
    sync::Arc,
    vec::Vec,
};

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt},
    net::UdpSocket,
    sync::RwLock,
    task,
};

use super::gain;
use super::repl;
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
        let mut repl = repl::App::new();

        let mut lines = tokio::io::BufReader::new(tokio::io::stdin()).lines();
        let mut stdout = tokio::io::stdout();

        loop {
            stdout.write_all(b"> ").await?;
            stdout.flush().await?;

            if let Some(line) = lines.next_line().await? {
                let command = repl.parse_command(line.as_ref())?;

                let cont = match command {
                    repl::Command::Nop => true,
                    repl::Command::Help => {
                        stdout.write_all(repl.help()).await?;
                        true
                    }
                    repl::Command::Exit => false,
                    repl::Command::Info => {
                        stdout.write_all(self.info().await.as_bytes()).await?;
                        true
                    }
                    repl::Command::Rm(index) => {
                        let mut tx_vec = self.tx_addrs.write().await;
                        if index < (*tx_vec).len() {
                            (*tx_vec).remove(index);
                        }
                        true
                    }
                    repl::Command::Add(addr) => {
                        if let Some(error) = self.add_tx_addrs(addr).await.err() {
                            stdout.write_all(error.to_string().as_bytes()).await?;
                        }
                        true
                    }
                    repl::Command::Error(mut e) => {
                        writeln!(e).unwrap();
                        stdout.write_all(e.as_bytes()).await?;
                        true
                    }
                };

                if !cont {
                    break;
                }
            } else {
                break;
            }
        }
        Ok(())
    }

    async fn info(&self) -> String {
        let mut b = String::with_capacity(1024);

        {
            let g = self.gain.read().await;
            writeln!(b, "gain: {}", g).unwrap();
        }

        writeln!(b, "rx addr: {}", self.rx_addr).unwrap();

        {
            for (i, t) in self.tx_addrs.read().await.iter().enumerate() {
                writeln!(b, "tx addr[{}]: {}", i, t).unwrap();
            }
        }

        b
    }
}
