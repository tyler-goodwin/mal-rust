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
  let key = match list.get(0) {
    Some(k) => k.symbol_value().unwrap(),
    None => return Err(MalError::unknown()),
  };
  let value = match list.get(1) {
    Some(k) => eval(k.to_owned(), env)?,
    None => return Err(MalError::unknown()),
  };
  env.set(&key, value.clone());
  Ok(value)
}

fn let_star(env: &mut Env, list: &mut Vec<MalType>) -> MalResult {
  let mut new_env = Env::new(Some(env.to_owned()));
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

fn eval_do(input: &mut Vec<MalType>, env: &mut Env) -> MalResult {
  let input = MalType::List(input.to_owned());
  let list = eval_ast(input, env)?
    .list_value()
    .ok_or(MalError::unknown())?;
  match list.last() {
    Some(last) => Ok(last.to_owned()),
    None => Err(MalError::unknown()),
  }
}

fn eval_if(input: &mut Vec<MalType>, env: &mut Env) -> MalResult {
  if input.len() < 2 {
    return Err(MalError::unknown());
  }

  let condition = eval(input[0].to_owned(), env)?;
  if condition.is_truthy() {
    return eval(input[1].to_owned(), env);
  } else {
    return match input.get(2) {
      Some(falsey) => eval(falsey.to_owned(), env),
      None => Ok(MalType::Nil),
    };
  }
}

fn eval_fn_star(input: &mut Vec<MalType>, env: &mut Env) -> MalResult {
  if input.len() < 2 {
    return Err(MalError::generic("Not enough args to fn*, expecting 2"));
  }
  let args = &input[0];
  if let Some(args) = args.list_value() {
    let body = input[1].clone();
    let lambda = MalType::Lambda(MalLambda {
      env: env.clone(),
      args,
      body: vec![body],
    });
    Ok(lambda)
  } else {
    Err(MalError::generic(&format!(
      "Expecting vector as first argument of fn*, but got: {:?}",
      args
    )))
  }
}

fn call_lambda(
  parent: Env,
  binds: Vec<MalType>,
  mut body: Vec<MalType>,
  args: Vec<MalType>,
) -> MalResult {
  let binds: Vec<String> = binds
    .into_iter()
    .filter_map(|val| val.symbol_value())
    .collect();
  let mut env = Env::new_with_bindings(Some(parent), binds, args);
  let expr = body.remove(0);
  eval(expr, &mut env)
}

fn eval_ast(input: MalType, env: &mut Env) -> MalResult {
  // println!("EVAL AST Input: {}", printer::print_str(&input, true));
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

fn eval(input: MalType, env: &mut Env) -> MalResult {
  // println!("EVAL Input: {}", printer::print_str(&input, true));
  if !input.is_list() {
    // println!("EVAL NOT LIST");
    eval_ast(input, env)
  } else if input.list_value().unwrap().len() == 0 {
    Ok(input)
  } else if is_special_form(&input) {
    // println!("EVAL Special Form");
    if let Some(mut list) = input.list_value() {
      if let MalType::Symbol(sym) = list.remove(0) {
        match sym.as_str() {
          "def!" => def(env, list),
          "let*" => let_star(env, &mut list),
          "do" => eval_do(&mut list, env),
          "if" => eval_if(&mut list, env),
          "fn*" => eval_fn_star(&mut list, env),
          _ => Err(MalError::generic("Unknown special symbol")),
        }
      } else {
        panic!("No longer a list somehow");
      }
    } else {
      panic!("No longer a list somehow");
    }
  } else {
    // Must be a function or lambda call
    let mut list = eval_ast(input, env)?.list_value().unwrap();
    match list.remove(0) {
      MalType::Function(MalFunc { func, .. }) => func(&mut list),
      MalType::Lambda(MalLambda {
        env, args, body, ..
      }) => {
        // println!("EVAL Calling lambda");
        call_lambda(env.clone(), args.clone(), body.clone(), list)
      }
      _ => return Err(MalError::not_a_function()),
    }
  }
}

fn is_special_form(input: &MalType) -> bool {
  if let Some(list) = input.list_value() {
    if list.len() == 0 {
      return false;
    }
    if let Some(sym) = list[0].symbol_value() {
      return match sym.as_str() {
        "def!" | "let*" | "do" | "if" | "fn*" => true,
        _ => false,
      };
    }
  }
  false
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
  // Add native lib functions
  for (sym, func) in &*core::CORE_FUNCTIONS {
    env.set(sym, MalType::Function(MalFunc { func: *func }))
  }
  // Eval stdlib mal functions
  let ast = reader::read_str(String::from("(def! not (fn* (a) (if a false true)))")).unwrap();
  eval(ast, &mut env).unwrap();

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
