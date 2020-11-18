mod gain;
mod runner;

use std::io::Result;
use std::net::ToSocketAddrs;
use tokio::runtime;

pub fn main<T>(rx_addr: T, tx_addrs: &[T], gain: f32) -> Result<()>
where
    T: ToSocketAddrs,
{
    let r = runner::Runner::new(rx_addr, gain)?;

    let rt = runtime::Builder::new_current_thread().enable_io().build()?;

    rt.block_on(async move {
        for t in tx_addrs {
            r.add_tx_addrs(t).await.unwrap();
        }

        r.spawn_pipe_loop().await?;

        r.repl().await
    })
}
