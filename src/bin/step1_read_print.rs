use std::io::{self, Write};

use mal_rust::types::*;
use mal_rust::{printer, reader};

enum ReadResult {
  InputRecv(MalType),
  Done(()),
}

enum RepResult {
  Continue,
  Done,
}

fn read() -> Result<ReadResult, MalError> {
  let mut input = String::new();
  print!("user> ");
  io::stdout().flush().expect("Could not flush stdout");

  let bytes = io::stdin()
    .read_line(&mut input)
    .expect("Could not read from stdin");
  let out = if bytes != 0 {
    ReadResult::InputRecv(reader::read_str(String::from(input.trim()))?)
  } else {
    ReadResult::Done(())
  };
  Ok(out)
}

fn eval(input: MalType) -> Result<MalType, MalError> {
  Ok(input)
}

fn print(output: MalType) {
  let out = printer::print_str(&output, true);
  println!("{}", out);
  io::stdout().flush().unwrap();
}

fn rep() -> Result<RepResult, MalError> {
  match read()? {
    ReadResult::InputRecv(input) => {
      print(eval(input)?);
      Ok(RepResult::Continue)
    }
    ReadResult::Done(_) => Ok(RepResult::Done),
  }
}

fn main() {
  loop {
    match rep() {
      Ok(RepResult::Continue) => (),
      Ok(RepResult::Done) => break,
      Err(e) => match e.reason() {
        MalErrorReason::BlankLine => (),
        _ => eprintln!("{}", e),
      },
    }
  }
  println!();
}
