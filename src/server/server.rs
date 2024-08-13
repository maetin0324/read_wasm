use async_ucx::ucp::*;
use std::mem::MaybeUninit;
use std::sync::atomic::*;
use anyhow::{Ok, Result};

pub async fn server() -> Result<()> {
    let context = Context::new()?;
    let worker = context.create_worker()?;

    tokio::task::spawn_local(worker.clone().polling());

    let mut listener = worker.create_listener("0.0.0.0:10000".parse().unwrap())?;
    println!("Listening on {}", listener.socket_addr().unwrap());
    for i in 0u8.. {
      let conn = listener.next().await;
      conn.remote_addr().unwrap();
      let ep = worker.accept(conn).await?;
    }
    Ok(())
}