use async_ucx::ucp::*;
use std::mem::MaybeUninit;
use anyhow::{Ok, Result};

pub async fn client(server_addr: String, data: Vec<u8>) -> Result<()> {
  let context = Context::new().unwrap();
  let worker = context.create_worker().unwrap();
  tokio::task::spawn_local(worker.clone().polling());
  let endpoint = worker
    .connect_socket(server_addr.parse().unwrap())
    .await?;
  endpoint.print_to_stderr();

  let mut id = [MaybeUninit::uninit()];
  endpoint.worker().tag_recv(100, &mut id).await?;
  let tag = unsafe { id[0].assume_init() } as u64 + 200;
  println!("client: got tag {:?}", tag);
  let len = data.len().to_le_bytes();
  endpoint.tag_send(tag, &len).await?;
  endpoint.tag_send(tag, &data).await?;
  endpoint
    .worker()
    .tag_recv(tag, &mut [MaybeUninit::uninit()])
    .await?;
  Ok(())
}

