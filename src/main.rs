use std::fs::File;
use std::io::{BufReader, Read, Write};
use clap::{command, Parser};
use read_wasm::binary::wasm::Wasm;
use read_wasm::exec::exec_machine::ExecMachine;
use read_wasm::exec::value::Value;
use read_wasm::exec::wasi::WasiSnapshotPreview1;

#[derive(Parser, Debug)]
#[command(author, about)]
struct Cli {
  #[command(subcommand)]
  subcmd: SubCommand,
}

#[derive(Parser, Debug)]
enum SubCommand {
  Run {
    filename: String,

    #[clap(short, long, default_value = "_start")]
    entry_point: String,

    #[clap(short, long)]
    locals: Vec<i64>,
  },
  Vm {
    filename: String,
  },
  Serialize {
    filename: String,

    #[clap(short, long, default_value = "_start")]
    entry_point: String,

    #[clap(short, long)]
    locals: Vec<i64>,
  },
  Server,
  Client {
    server_addr: String,
    filename: String,
  },
}

#[tokio::main]
async fn main() {
  let args = Cli::parse();
  match args.subcmd {
    SubCommand::Run { filename, entry_point, locals } => {
      let file = File::open(filename).unwrap();
      let wasm = Wasm::new(BufReader::new(file));

      let locals = Value::parse_from_i64_vec(locals);

      let mut machine = ExecMachine::init(wasm, &entry_point, locals);
      let mut wasi = WasiSnapshotPreview1::new();
      match machine.exec(&mut wasi).await {
        Ok(_) => { println!("return {:?}", machine.value_stack.last()); },
        Err(e) => {
          println!("ExecuteError: {:?}", e.message);
        },
      }
    }
    SubCommand::Vm { filename } => {
      let mut file = File::open(filename).unwrap();
      let mut se  = Vec::new();
      file.read_to_end(&mut se).unwrap();
      let mut machine = ExecMachine::deserialize(&se).await.unwrap();
      let mut wasi = WasiSnapshotPreview1::new();
      match machine.exec(&mut wasi).await {
        Ok(_) => { println!("return {:?}", machine.value_stack.last()); },
        Err(e) => {
          println!("ExecuteError: {:?}", e.message);
          println!("VM: {:#?}", e.vm);
        },
      }
    }
    SubCommand::Serialize { filename, entry_point, locals } => {
      let file = File::open(filename).unwrap();
      let wasm = Wasm::new(BufReader::new(file));
      let locals = Value::parse_from_i64_vec(locals);

      let machine = ExecMachine::init(wasm, &entry_point, locals);
      let data = machine.serialize_vm();
      File::create("vm.serialized").unwrap().write_all(&data).unwrap();
    }
    #[cfg(feature = "ucx")]
    SubCommand::Server => {
      let local = tokio::task::LocalSet::new();
      local.run_until(
      read_wasm::comm::server::server_start()
      ).await.unwrap();
    }
    #[cfg(feature = "ucx")]
    SubCommand::Client { server_addr, filename } => {
      let mut file = File::open(filename).unwrap();
      let mut data  = Vec::new();
      file.read_to_end(&mut data).unwrap();
      let local = tokio::task::LocalSet::new();
      local.run_until(
        read_wasm::comm::client::client(server_addr, data)
      ).await.unwrap();
    }
    _ => {}
  }
}
