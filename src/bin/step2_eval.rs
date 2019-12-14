use std::collections::HashMap;
use std::io::{self, Write};

use mal_rust::core;
use mal_rust::types::*;
use mal_rust::{printer, reader};

type ReplEnv = HashMap<String, MalType>;

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

fn lookup(env: &ReplEnv, sym: &str) -> Result<MalType, MalError> {
  if let Some(value) = env.get(sym) {
    Ok(value.to_owned())
  } else {
    Err(MalError::symbol_not_found(sym.to_string()))
  }
}

fn eval_ast(input: MalType, env: &mut ReplEnv) -> Result<MalType, MalError> {
  let mut eval_list = |list: Vec<MalType>| -> Result<Vec<MalType>, MalError> {
    list.into_iter().map(|v| eval(v.clone(), env)).collect()
  };
  let value = match input {
    MalType::Symbol(sym) => lookup(env, &sym)?,
    MalType::List(list) => MalType::List(eval_list(list)?),
    MalType::Vector(list) => MalType::Vector(eval_list(list)?),
    MalType::HashMap(list) => MalType::HashMap(eval_list(list)?),
    _ => input,
  };
  Ok(value)
}

fn eval(input: MalType, env: &mut ReplEnv) -> Result<MalType, MalError> {
  if !input.is_list() {
    return eval_ast(input, env);
  }
  if let Some(list) = input.list_value() {
    if list.len() == 0 {
      return Ok(input);
    }
  }
  let new_input = eval_ast(input, env)?;
  if let Some(mut list) = new_input.list_value() {
    let result = match list.remove(0) {
      MalType::Function(malfunc) => (malfunc.func)(&mut list)?,
      _ => return Err(MalError::not_a_function()),
    };
    Ok(result)
  } else {
    Err(MalError::unknown())
  }
}

fn print(output: MalType) {
  let out = printer::print_str(&output);
  println!("{}", out);
  io::stdout().flush().unwrap();
}

fn rep(env: &mut ReplEnv) -> Result<RepResult, MalError> {
  match read()? {
    ReadResult::InputRecv(input) => {
      print(eval(input, env)?);
      Ok(RepResult::Continue)
    }
    ReadResult::Done(_) => Ok(RepResult::Done),
  }
}

fn main() {
  let mut env = HashMap::new();
  env.insert(
    "+".to_string(),
    MalType::Function(MalFunc { func: core::plus }),
  );
  env.insert(
    "-".to_string(),
    MalType::Function(MalFunc { func: core::minus }),
  );
  env.insert(
    "*".to_string(),
    MalType::Function(MalFunc {
      func: core::multiply,
    }),
  );
  env.insert(
    "/".to_string(),
    MalType::Function(MalFunc { func: core::divide }),
  );

  loop {
    match rep(&mut env) {
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
