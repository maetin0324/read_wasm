use std::fs::File;
use std::io::{BufReader, Read};
use clap::{command, Parser};
use read_wasm::binary::wasm::Wasm;
use read_wasm::exec::exec_machine::ExecMachine;
use read_wasm::exec::value::Value;

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
}
#[tokio::main]
async fn main() {
  let args = Cli::parse();
  match args.subcmd {
    SubCommand::Run { filename, entry_point, locals } => {
      let file = File::open(filename).unwrap();
      let wasm = Wasm::new(BufReader::new(file));
      // println!("{:#?}", wasm);

      let locals = Value::parse_from_i64_vec(locals);

      let mut machine = ExecMachine::init(wasm, &entry_point, locals);
      match machine.exec().await {
        Ok(_) => { println!("return {:?}", machine.value_stack.last()); },
        Err(e) => {
          println!("ExecuteError: {:?}", e.message);
          println!("VM: {:#?}", e.vm);
        },
      }
    }
    SubCommand::Vm { filename } => {
      let mut file = File::open(filename).unwrap();
      let mut se  = Vec::new();
      file.read_to_end(&mut se).unwrap();
      let mut machine = ExecMachine::deserialize(&se).await.unwrap();
      println!("{:#?}", machine);
      match machine.exec().await {
        Ok(_) => { println!("return {:?}", machine.value_stack.last()); },
        Err(e) => {
          println!("ExecuteError: {:?}", e.message);
          println!("VM: {:#?}", e.vm);
        },
      }
      // let byte = unsafe {any_as_u8_slice(&machine)};
      // println!("{:?}", byte.len());
      // let (_head, body, _tail) = unsafe { byte.align_to::<ExecMachine>()};
      // let machine = body[0].clone();

      
    }
    SubCommand::Serialize { filename, entry_point, locals } => {
      let file = File::open(filename).unwrap();
      let wasm = Wasm::new(BufReader::new(file));
      let locals = Value::parse_from_i64_vec(locals);

      let machine = ExecMachine::init(wasm, &entry_point, locals);
      machine.serialize_vm();
    }
  }
}
