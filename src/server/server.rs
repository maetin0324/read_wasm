use async_ucx::ucp::*;
use std::mem::MaybeUninit;

use crate::exec::exec_machine::ExecMachine;

pub async fn server_start() -> anyhow::Result<()> {
    let context = Context::new()?;
    let worker = context.create_worker()?;

    tokio::task::spawn_local(worker.clone().polling());

    let mut listener = worker.create_listener("0.0.0.0:10000".parse().unwrap())?;
    println!("Listening on {}", listener.socket_addr().unwrap());
    for i in 0u8.. {
      let conn = listener.next().await;
      conn.remote_addr().unwrap();
      let ep = worker.accept(conn).await?;

      println!("accept: {}", i);
      ep.tag_send(100, &[i]).await.unwrap();
      tokio::task::spawn_local(async move {
        let tag = i as u64 + 200;
        let mut len_buf = vec![MaybeUninit::uninit(); 8];
        loop {
          ep.worker().tag_recv(tag, &mut len_buf).await.unwrap();
          let len = usize::from_le_bytes(unsafe {
              len_buf.iter().map(|x| x.assume_init()).collect::<Vec<u8>>().try_into().unwrap()
          });
          let mut buf = vec![MaybeUninit::uninit(); len];
      
          ep.worker().tag_recv(tag, &mut buf).await.unwrap();
          let buf = unsafe {
            buf.iter().map(|x| x.assume_init()).collect::<Vec<u8>>()
          };
          println!("len: {}, buf: {:?}", len, buf);
          ep.tag_send(tag, &[0]).await.unwrap();
          let mut machine = ExecMachine::deserialize(&buf).await.unwrap();
          // println!("{:#?}", machine);
          match machine.exec().await {
            std::result::Result::Ok(_) => { println!("return {:?}", machine.value_stack.last()); },
            Err(e) => {
              println!("ExecuteError: {:?}", e.message);
              println!("VM: {:#?}", e.vm);
            },
          }
        }
      });
    }
    anyhow::Ok(())
}

