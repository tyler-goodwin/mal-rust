use mal_rust::core;
use mal_rust::env::Env;
use mal_rust::types::*;
use mal_rust::{printer, reader};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::{env, process};

fn read(input: &str) -> MalResult {
  reader::read_str(input.to_string())
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

fn let_star_env(env: &mut Env, list: &mut Vec<MalType>) -> Result<Env, MalError> {
  let mut new_env = Env::new(Some(env.to_owned()));
  let mut list = list.clone();
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
  Ok(new_env)
}

fn eval_do(input: &mut Vec<MalType>, env: &mut Env) -> MalResult {
  let list = MalType::List(input[..(input.len() - 1)].to_vec());
  let _list = eval_ast(list, env)?
    .list_value()
    .ok_or(MalError::unknown())?;
  match input.last() {
    Some(last) => Ok(last.to_owned()),
    None => Err(MalError::unknown()),
  }
}

fn eval_if(input: &mut Vec<MalType>, env: &mut Env) -> MalResult {
  if input.len() < 2 {
    return Err(MalError::unknown());
  }

  let condition = eval(input[0].to_owned(), env)?;
  let result = if condition.is_truthy() {
    input[1].to_owned()
  } else {
    match input.get(2) {
      Some(falsey) => falsey.to_owned(),
      None => MalType::Nil,
    }
  };
  Ok(result)
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
  let mut input = input.clone();
  let mut env = env.clone();
  loop {
    if !input.is_list() {
      return eval_ast(input, &mut env);
    } else if input.list_value().unwrap().len() == 0 {
      return Ok(input);
    } else if is_special_form(&input) {
      if let Some(mut list) = input.list_value() {
        if let MalType::Symbol(sym) = list.remove(0) {
          match sym.as_str() {
            "def!" => {
              return def(&mut env, list);
            }
            "let*" => {
              env = let_star_env(&mut env, &mut list)?;
              input = list.get(1).unwrap().clone();
            }
            "do" => {
              input = eval_do(&mut list, &mut env)?;
            }
            "if" => {
              input = eval_if(&mut list, &mut env)?;
            }
            "fn*" => {
              return eval_fn_star(&mut list, &mut env);
            }
            "quote" => {
              if list.len() < 1 {
                return Err(MalError::wrong_arguments("Missing argument for quote"));
              }
              return Ok(list.remove(0));
            }
            "quasiquote" => input = quasiquote(&mut list)?,
            _ => return Err(MalError::generic("Unknown special symbol")),
          };
        } else {
          panic!("No longer a list somehow");
        }
      } else {
        panic!("No longer a list somehow");
      }
    } else {
      // Must be a function or lambda call
      let mut list = eval_ast(input, &mut env)?.list_value().unwrap();
      match list.remove(0) {
        MalType::Function(MalFunc { func, env, .. }) => return func(&mut list, env),
        MalType::Lambda(MalLambda {
          env: l_env,
          args,
          body,
          ..
        }) => {
          let binds: Vec<String> = args
            .into_iter()
            .filter_map(|val| val.symbol_value())
            .collect();
          env = Env::new_with_bindings(Some(l_env), binds, list);
          input = body.get(0).unwrap().clone();
        }
        _ => return Err(MalError::not_a_function()),
      };
    }
  }
}

fn quasiquote(args: &mut Vec<MalType>) -> MalResult {
  if args.len() == 0 {
    return Ok(MalType::List(vec![]));
  }
  let ast = args.remove(0);
  if !ast.is_pair() {
    Ok(MalType::List(vec![
      MalType::Symbol("quote".to_string()),
      ast.clone(),
    ]))
  } else if first(&ast).is_symbol_named("unquote") {
    Ok(first(&rest(&ast)))
  } else if first(&ast).is_pair() && first(&first(&ast)).is_symbol_named("splice-unquote") {
    let ret_list = vec![
      MalType::Symbol("concat".to_string()),
      first(&rest(&first(&ast))),
      quasiquote(&mut vec![rest(&ast)])?,
    ];
    Ok(MalType::List(ret_list))
  } else {
    let ret_list = vec![
      MalType::Symbol("cons".to_string()),
      quasiquote(&mut vec![first(&ast)])?,
      quasiquote(&mut vec![rest(&ast)])?,
    ];
    Ok(MalType::List(ret_list))
  }
}

fn first(val: &MalType) -> MalType {
  let list = val.list_value().expect("Requires a list");
  assert!(list.len() > 0);
  list[0].clone()
}

fn rest(val: &MalType) -> MalType {
  let list = val.list_value().expect("Requires a list");
  assert!(list.len() > 0);
  MalType::List(list[1..].to_owned())
}

fn is_special_form(input: &MalType) -> bool {
  if let Some(list) = input.list_value() {
    if list.len() == 0 {
      return false;
    }
    if let Some(sym) = list[0].symbol_value() {
      return match sym.as_str() {
        "def!" | "let*" | "do" | "if" | "fn*" | "quote" | "quasiquote" => true,
        _ => false,
      };
    }
  }
  false
}

fn eval_fn(args: &mut Vec<MalType>, env: Option<Env>) -> MalResult {
  if let Some(arg) = args.get(0) {
    let mut env = env.expect("No env provided");
    eval(arg.to_owned(), &mut env)
  } else {
    Err(MalError::generic("Not enough arguments"))
  }
}

fn print(output: MalType) -> String {
  printer::print_str(&output, true)
}

fn rep(input: String, env: &mut Env) -> Result<String, MalError> {
  let out = read(&input)?;
  let out = print(eval(out, env)?);
  Ok(out)
}

fn main() {
  let mut env = Env::new(None);
  // Add native lib functions
  for (sym, func) in &*core::CORE_FUNCTIONS {
    env.set(
      sym,
      MalType::Function(MalFunc {
        func: *func,
        env: None,
      }),
    )
  }
  env.set(
    "eval",
    MalType::Function(MalFunc {
      func: eval_fn,
      env: Some(env.clone()),
    }),
  );

  env.set("*ARGV*", MalType::List(vec![]));
  // Eval stdlib mal functions
  let ast = reader::read_str(String::from("(def! not (fn* (a) (if a false true)))")).unwrap();
  eval(ast, &mut env).unwrap();

  let ast = reader::read_str(
    "(def! load-file (fn* (f) (eval (read-string (str \"(do \" (slurp f) \"\nnil)\")))))"
      .to_string(),
  )
  .unwrap();
  eval(ast, &mut env).unwrap();

  let mut rl = Editor::<()>::new();
  match rl.load_history(".mal-history") {
    _ => {}
  }

  let mut args: Vec<String> = env::args().collect();
  if args.len() > 1 {
    args.remove(0); // Remove name of executable
    let file = args.remove(0);
    env.set(
      "*ARGV*",
      MalType::List(args.iter().map(|it| MalType::String(it.clone())).collect()),
    );
    let result = rep(format!("(load-file \"{}\")", file), &mut env);
    match result {
      Err(err) => {
        eprintln!("{}", err);
        process::exit(1);
      }
      _ => process::exit(0),
    }
  }

  loop {
    let readline = rl.readline("user> ");
    match readline {
      Ok(line) => {
        rl.add_history_entry(line.as_str());
        match rep(line, &mut env) {
          Ok(out) => println!("{}", out),
          Err(err) => match err.reason() {
            MalErrorReason::BlankLine => (),
            _ => eprintln!("{}", err),
          },
        }
      }
      Err(ReadlineError::Interrupted) => {
        // Do nothing
      }
      Err(ReadlineError::Eof) => break,
      Err(err) => {
        println!("Error: {:?}", err);
        break;
      }
    }
  }
  rl.save_history(".mal-history").unwrap();
}
