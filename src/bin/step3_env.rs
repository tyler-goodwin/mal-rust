use std::io::{self, Write};

use mal_rust::core;
use mal_rust::env::Env;
use mal_rust::types::*;
use mal_rust::{printer, reader};

enum ReadResult {
  InputRecv(MalType),
  Done,
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
    ReadResult::Done
  };
  Ok(out)
}

fn eval_hash_map(list: Vec<MalType>, env: &mut Env) -> Result<Vec<MalType>, MalError> {
  let mut new_list = Vec::new();
  for (i, item) in list.iter().enumerate() {
    if i % 2 != 0 {
      new_list.push(eval(item.to_owned(), env)?);
    } else {
      new_list.push(item.to_owned());
    }
  }
  Ok(new_list)
}

fn def(env: &mut Env, list: Vec<MalType>) -> MalResult {
  let key = match list.get(1) {
    Some(k) => k.symbol_value().unwrap(),
    None => return Err(MalError::unknown()),
  };
  let value = match list.get(2) {
    Some(k) => eval(k.to_owned(), env)?,
    None => return Err(MalError::unknown()),
  };
  env.set(&key, value.clone());
  Ok(value)
}

fn let_star(env: &mut Env, list: &mut Vec<MalType>) -> MalResult {
  let mut new_env = Env::new(Some(env.to_owned()));
  list.remove(0); // Remove "let*"
  let mut bindings = match list.remove(0) {
    MalType::List(list) | MalType::Vector(list) => list,
    _ => return Err(MalError::wrong_arguments("Expected list or vector")),
  };

  if bindings.len() % 2 != 0 {
    return Err(MalError::unknown());
  }

  loop {
    if bindings.len() == 0 {
      break;
    }
    let symbol = match bindings.remove(0) {
      MalType::Symbol(sym) => sym,
      _ => return Err(MalError::wrong_arguments("Expected symbol")),
    };

    let value = eval(bindings.remove(0), &mut new_env)?;
    new_env.set(&symbol, value);
  }

  Ok(eval(list.remove(0), &mut new_env)?)
}

fn eval_ast(input: MalType, env: &mut Env) -> Result<MalType, MalError> {
  let mut eval_list = |list: Vec<MalType>| -> Result<Vec<MalType>, MalError> {
    list.into_iter().map(|v| eval(v.clone(), env)).collect()
  };
  let value = match input {
    MalType::Symbol(sym) => env.get(&sym)?,
    MalType::List(list) => MalType::List(eval_list(list)?),
    MalType::Vector(list) => MalType::Vector(eval_list(list)?),
    MalType::HashMap(list) => MalType::HashMap(eval_hash_map(list, env)?),
    _ => input,
  };
  Ok(value)
}

fn eval(input: MalType, env: &mut Env) -> Result<MalType, MalError> {
  if !input.is_list() {
    return eval_ast(input, env);
  }
  if let Some(list) = input.list_value() {
    if list.len() == 0 {
      return Ok(input);
    }
  }

  if let Some(mut list) = input.list_value() {
    let result = match &list[0] {
      MalType::Symbol(sym) => match sym.as_str() {
        "def!" => def(env, list)?,
        "let*" => let_star(env, &mut list)?,
        _ => {
          let mut list = eval_ast(input, env)?.list_value().unwrap();
          match list.remove(0) {
            MalType::Function(malfunc) => (malfunc.func)(&mut list)?,
            _ => return Err(MalError::not_a_function()),
          }
        }
      },
      _ => return Err(MalError::unknown()),
    };
    Ok(result)
  } else {
    Err(MalError::unknown())
  }
}

fn print(output: MalType) {
  let out = printer::print_str(&output, true);
  println!("{}", out);
  io::stdout().flush().unwrap();
}

fn rep(env: &mut Env) -> Result<RepResult, MalError> {
  match read()? {
    ReadResult::InputRecv(input) => {
      print(eval(input, env)?);
      Ok(RepResult::Continue)
    }
    ReadResult::Done => Ok(RepResult::Done),
  }
}

fn main() {
  let mut env = Env::new(None);
  env.set("+", MalType::Function(MalFunc { func: core::plus }));
  env.set("-", MalType::Function(MalFunc { func: core::minus }));
  env.set("/", MalType::Function(MalFunc { func: core::divide }));
  env.set(
    "*",
    MalType::Function(MalFunc {
      func: core::multiply,
    }),
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
