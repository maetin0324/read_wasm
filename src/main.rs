use std::fs::File;
use std::io::BufReader;
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

    #[clap(short, long)]
    locals: Vec<i64>,
}

fn main() {
    let args = Cli::parse();
    let filename = args.filename;
    let entry_point = args.entry_point;
    
    let file = File::open(filename).unwrap();
    let wasm = Wasm::new(BufReader::new(file));
    println!("{:#?}", wasm);

    let locals = Value::parse_from_i64_vec(args.locals);

    let mut machine = ExecMachine::new();
    match machine.exec(wasm, &entry_point, locals) {
        Ok(_) => { println!("return {:?}", machine.value_stack.last()); },
        Err(e) => {
            println!("ExecuteError: {:?}", e.message);
            println!("VM: {:#?}", e.vm);
        },
    }
}
