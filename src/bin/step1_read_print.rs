use std::error::Error;
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

fn read() -> Result<ReadResult, Box<dyn Error>> {
  let mut input = String::new();
  print!("user> ");
  io::stdout().flush()?;

  let bytes = io::stdin().read_line(&mut input)?;
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
  let out = printer::print_str(&output);
  println!("{}", out);
  io::stdout().flush().unwrap();
}

fn rep() -> Result<RepResult, Box<dyn Error>> {
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
      Err(e) => eprintln!("Error: {}", e),
    }
  }
  println!();
}
