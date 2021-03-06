use crate::env::Env;
use crate::printer;
use crate::reader;
use crate::types::*;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fs;

macro_rules! gen_functions_map {
  ($( $sym:expr => $func:ident ),*) => {
    {
      let mut map = HashMap::new();
      $(
        map.insert($sym, $func as CoreFunction);
      )*
      map
    }
  };
}

lazy_static! {
  pub static ref CORE_FUNCTIONS: HashMap<&'static str, CoreFunction> = {
    gen_functions_map! {
      "+" => plus,
      "-" => minus,
      "*" => multiply,
      "/" => divide,
      "list" => list,
      "list?" => is_list,
      "empty?" => is_empty,
      "count" => count,
      "=" => equal,
      "<" => less_than,
      "<=" => less_than_or_eq,
      ">" => greater_than,
      ">=" => greater_than_or_eq,
      "prn" => prn,
      "println" => println,
      "pr-str" => pr_str,
      "str" => str,
      "read-string" => read_string,
      "slurp" => slurp,
      "atom" => atom,
      "atom?" => is_atom,
      "deref" => deref,
      "reset!" => reset,
      "swap!" => swap,
      "cons" => cons,
      "concat" => concat
    }
  };
}

pub fn plus(args: &mut Vec<MalType>, _env: Option<Env>) -> MalResult {
  let result = to_numbers(args)?.iter().fold(0, |acc, x| acc + x);
  Ok(MalType::Number(result))
}

pub fn minus(args: &mut Vec<MalType>, _env: Option<Env>) -> MalResult {
  let mut args = to_numbers(args)?;
  let mut result = args.remove(0);
  for i in args {
    result -= i;
  }
  Ok(MalType::Number(result))
}

pub fn multiply(args: &mut Vec<MalType>, _env: Option<Env>) -> MalResult {
  let result = to_numbers(args)?.iter().fold(1, |acc, x| acc * x);
  Ok(MalType::Number(result))
}

pub fn divide(args: &mut Vec<MalType>, _env: Option<Env>) -> MalResult {
  let mut args = to_numbers(args)?;
  let mut result = args.remove(0);
  for i in args {
    result /= i;
  }
  Ok(MalType::Number(result))
}

pub fn list(args: &mut Vec<MalType>, _env: Option<Env>) -> MalResult {
  Ok(MalType::List(args.to_owned()))
}

pub fn is_list(args: &mut Vec<MalType>, _env: Option<Env>) -> MalResult {
  if args.len() == 0 {
    return Ok(MalType::False);
  }

  Ok(MalType::to_bool(args[0].is_list()))
}

pub fn is_empty(args: &mut Vec<MalType>, _env: Option<Env>) -> MalResult {
  expected_arguments(args, 1)?;
  match args[0].list_value() {
    Some(list) => Ok(MalType::to_bool(list.len() == 0)),
    None => Err(MalError::wrong_arguments("Not a list")),
  }
}

pub fn count(args: &mut Vec<MalType>, _env: Option<Env>) -> MalResult {
  expected_arguments(args, 1)?;
  if args[0].is_nil() {
    return Ok(MalType::Number(0));
  }
  match args[0].list_value() {
    Some(list) => Ok(MalType::Number(list.len().try_into().unwrap())),
    None => Err(MalError::wrong_arguments("Not a list")),
  }
}

pub fn equal(args: &mut Vec<MalType>, _env: Option<Env>) -> MalResult {
  expected_arguments(args, 2)?;
  Ok(MalType::to_bool(values_equal(&args[0], &args[1])))
}

pub fn less_than(args: &mut Vec<MalType>, _env: Option<Env>) -> MalResult {
  expected_arguments(args, 2)?;
  let first = get_number(&args[0])?;
  let second = get_number(&args[1])?;
  Ok(MalType::to_bool(first < second))
}

pub fn less_than_or_eq(args: &mut Vec<MalType>, _env: Option<Env>) -> MalResult {
  expected_arguments(args, 2)?;
  let first = get_number(&args[0])?;
  let second = get_number(&args[1])?;
  Ok(MalType::to_bool(first <= second))
}

pub fn greater_than(args: &mut Vec<MalType>, _env: Option<Env>) -> MalResult {
  expected_arguments(args, 2)?;
  let first = get_number(&args[0])?;
  let second = get_number(&args[1])?;
  Ok(MalType::to_bool(first > second))
}

pub fn greater_than_or_eq(args: &mut Vec<MalType>, _env: Option<Env>) -> MalResult {
  expected_arguments(args, 2)?;
  let first = get_number(&args[0])?;
  let second = get_number(&args[1])?;
  Ok(MalType::to_bool(first >= second))
}

pub fn prn(args: &mut Vec<MalType>, _env: Option<Env>) -> MalResult {
  print(args, true)?;
  Ok(MalType::Nil)
}

pub fn println(args: &mut Vec<MalType>, _env: Option<Env>) -> MalResult {
  print(args, false)?;
  Ok(MalType::Nil)
}

pub fn pr_str(args: &mut Vec<MalType>, _env: Option<Env>) -> MalResult {
  Ok(MalType::String(join(args, " ", true)))
}

pub fn str(args: &mut Vec<MalType>, _env: Option<Env>) -> MalResult {
  Ok(MalType::String(join(args, "", false)))
}

pub fn read_string(args: &mut Vec<MalType>, _env: Option<Env>) -> MalResult {
  expected_arguments(args, 1)?;
  let arg = args.get(0).expect("Somehow lost an argument");
  if let Some(arg) = arg.string_value() {
    reader::read_str(arg)
  } else {
    Err(MalError::generic("Not a string"))
  }
}

pub fn slurp(args: &mut Vec<MalType>, _env: Option<Env>) -> MalResult {
  expected_arguments(args, 1)?;
  let arg = args.get(0).expect("Somehow lost an argument");
  if let Some(arg) = arg.string_value() {
    match fs::read_to_string(arg) {
      Ok(contents) => Ok(MalType::String(contents)),
      Err(err) => Err(MalError::generic(&err.to_string())),
    }
  } else {
    Err(MalError::generic("Not a string"))
  }
}

pub fn atom(args: &mut Vec<MalType>, _env: Option<Env>) -> MalResult {
  expected_arguments(args, 1)?;
  let arg = args.get(0).expect("Somehow lost an argument");
  Ok(MalType::atom(arg.to_owned()))
}

pub fn is_atom(args: &mut Vec<MalType>, _env: Option<Env>) -> MalResult {
  expected_arguments(args, 1)?;
  let arg = args.get(0).expect("Somehow lost an argument");
  Ok(MalType::to_bool(arg.is_atom()))
}

pub fn deref(args: &mut Vec<MalType>, _env: Option<Env>) -> MalResult {
  expected_arguments(args, 1)?;
  let arg = args.get(0).expect("Somehow lost an argument");
  match arg {
    MalType::Atom(value) => Ok(value.borrow().to_owned()),
    _ => Err(MalError::wrong_arguments("Not an atom")),
  }
}

pub fn reset(args: &mut Vec<MalType>, _env: Option<Env>) -> MalResult {
  expected_arguments(args, 2)?;
  let atom = args.remove(0);
  let value = args.remove(0);
  match atom {
    MalType::Atom(atom) => atom.replace(value.clone()),
    _ => return Err(MalError::wrong_arguments("Not an atom")),
  };
  Ok(value)
}

pub fn swap(args: &mut Vec<MalType>, _env: Option<Env>) -> MalResult {
  expected_arguments(args, 2)?;
  let mut atom = args.remove(0);
  let func = args.remove(0);
  atom.swap(func, args)
}

pub fn cons(args: &mut Vec<MalType>, _env: Option<Env>) -> MalResult {
  expected_arguments(args, 2)?;
  let value = args.remove(0);
  let list = args.remove(0);
  if let Some(list) = list.list_value() {
    let mut list = list.clone();
    list.insert(0, value);
    Ok(MalType::List(list))
  } else {
    Err(MalError::wrong_arguments("2nd argument was not a list"))
  }
}

pub fn concat(args: &mut Vec<MalType>, _env: Option<Env>) -> MalResult {
  let mut outlist = vec![];
  for arg in args {
    let mut list = vec_value(arg)?;
    outlist.append(&mut list);
  }
  Ok(MalType::List(outlist))
}

// ============================================================================
// Utilities
// ============================================================================
fn eval(mut args: &mut Vec<MalType>, env: &Env) -> MalResult {
  if let Some(MalFunc { func, .. }) = env
    .get("eval")
    .expect("eval not a function")
    .function_value()
  {
    func(&mut args, Some(env.clone()))
  } else {
    Err(MalError::generic("Not a function"))
  }
}

pub fn eval_func(func: MalType, args: &mut Vec<MalType>) -> MalResult {
  match func {
    MalType::Function(MalFunc { func, env, .. }) => func(args, env),
    MalType::Lambda(MalLambda {
      env,
      args: binds,
      body,
      ..
    }) => {
      let binds: Vec<String> = binds
        .into_iter()
        .filter_map(|val| val.symbol_value())
        .collect();
      let inner_env = Env::new_with_bindings(Some(env), binds, args.clone());
      eval(&mut body.clone(), &inner_env)
    }
    _ => Err(MalError::wrong_arguments("Not a function")),
  }
}

fn expected_arguments(args: &mut Vec<MalType>, expected: usize) -> Result<(), MalError> {
  if args.len() < expected {
    let msg = format!("Wrong number of args, expected {}", expected);
    Err(MalError::wrong_arguments(&msg))
  } else {
    Ok(())
  }
}

fn to_numbers(args: &mut Vec<MalType>) -> Result<Vec<i64>, MalError> {
  let mut results = Vec::new();
  for i in args {
    if let Some(val) = i.number_value() {
      results.push(val);
    } else {
      return Err(MalError::not_a_number());
    }
  }
  Ok(results)
}

fn get_number(arg: &MalType) -> Result<i64, MalError> {
  match arg.number_value() {
    Some(i) => Ok(i),
    None => Err(MalError::not_a_number()),
  }
}

fn values_equal(first: &MalType, second: &MalType) -> bool {
  use MalType::*;
  if first.is_list_or_vector() && second.is_list_or_vector() {
    let first = first.list_value().unwrap();
    let second = second.list_value().unwrap();
    list_equal(&first, &second)
  } else {
    match (first, second) {
      (Nil, Nil) => true,
      (True, True) => true,
      (False, False) => true,
      (Symbol(a), Symbol(b)) => a == b,
      (Number(a), Number(b)) => a == b,
      (String(a), String(b)) => a == b,
      (Keyword(a), Keyword(b)) => a == b,
      (List(a), List(b)) => list_equal(a, b),
      (Vector(a), Vector(b)) => list_equal(a, b),
      (HashMap(a), HashMap(b)) => list_equal(a, b),
      _ => false,
    }
  }
}

fn list_equal(first: &Vec<MalType>, second: &Vec<MalType>) -> bool {
  if first.len() != second.len() {
    return false;
  }
  let mut result = true;
  for (i, _) in first.iter().enumerate() {
    if !values_equal(&first[i], &second[i]) {
      result = false
    }
  }
  result
}

fn join(args: &mut Vec<MalType>, separator: &str, readable: bool) -> String {
  args
    .iter()
    .map(|val| printer::print_str(val, readable))
    .collect::<Vec<String>>()
    .join(separator)
}

fn print(args: &mut Vec<MalType>, readable: bool) -> MalResult {
  let joined = join(args, " ", readable);
  println!("{}", joined);
  Ok(MalType::Nil)
}

fn vec_value(arg: &MalType) -> Result<Vec<MalType>, MalError> {
  if let Some(v) = arg.list_value() {
    Ok(v.clone())
  } else {
    Err(MalError::wrong_arguments(&format!(
      "Expected a list or vector but got {:?}",
      arg
    )))
  }
}
