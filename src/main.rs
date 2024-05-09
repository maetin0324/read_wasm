use std::fs::File;
use clap::{command, Parser};
use read_wasm::binary::wasm::Wasm;
use read_wasm::exec::exec_machine::ExecMachine;
use read_wasm::exec::value::Value;

#[derive(Parser, Debug)]
#[command(author, about)]
struct Cli {
    filename: String,

    #[clap(short, long, default_value = "_start")]
    entry_point: String,
}

fn main() {
    let args = Cli::parse();
    let filename = args.filename;
    let entry_point = args.entry_point;
    
    let file = File::open(filename).unwrap();
    let wasm = Wasm::new(file);
    println!("{:?}", wasm);

    let mut machine = ExecMachine::new();
    match machine.exec(wasm, &entry_point, vec![Value::I64(1), Value::I64(2)]) {
        Ok(_) => { println!("return {:?}", machine.value_stack.last()); },
        Err(e) => {
            println!("ExecuteError: {:?}", e.message);
            println!("VM: {:?}", e.vm);
        },
    }
}
