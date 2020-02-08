use std::cell::RefCell;
use std::rc::Rc;
use std::{error, fmt};

use crate::core::eval_func;
use crate::env::*;

#[derive(Debug, Clone)]
pub enum MalType {
  Nil,
  True,
  False,
  Symbol(String),
  Number(i64),
  String(String),
  Keyword(String),
  List(Vec<MalType>),
  Vector(Vec<MalType>),
  HashMap(Vec<MalType>),
  Function(MalFunc),
  Lambda(MalLambda),
  Atom(Rc<RefCell<MalType>>),
}

impl MalType {
  pub fn atom(value: MalType) -> MalType {
    MalType::Atom(Rc::new(RefCell::new(value)))
  }

  pub fn swap(&mut self, func: MalType, args: &mut Vec<MalType>) -> MalResult {
    match self {
      MalType::Atom(ref atom) => {
        args.insert(0, atom.borrow().to_owned());
        let result = eval_func(func, args)?;
        atom.replace(result.clone());
        Ok(result)
      }
      _ => return Err(MalError::wrong_arguments("Not an atom")),
    }
  }

  pub fn is_list(&self) -> bool {
    match self {
      MalType::List(_) => true,
      _ => false,
    }
  }

  pub fn is_list_or_vector(&self) -> bool {
    match self {
      MalType::List(_) => true,
      MalType::Vector(_) => true,
      _ => false,
    }
  }
  pub fn is_map(&self) -> bool {
    match self {
      MalType::HashMap(_) => true,
      _ => false,
    }
  }

  pub fn is_truthy(&self) -> bool {
    match self {
      MalType::False => false,
      MalType::Nil => false,
      _ => true,
    }
  }

  pub fn is_nil(&self) -> bool {
    match self {
      MalType::Nil => true,
      _ => false,
    }
  }

  pub fn is_atom(&self) -> bool {
    match self {
      MalType::Atom(_) => true,
      _ => false,
    }
  }

  pub fn list_value(&self) -> Option<Vec<MalType>> {
    match self {
      MalType::List(list) => Some(list.to_owned()),
      MalType::Vector(list) => Some(list.to_owned()),
      _ => None,
    }
  }

  pub fn number_value(&self) -> Option<i64> {
    match self {
      MalType::Number(n) => Some(*n),
      _ => None,
    }
  }

  pub fn symbol_value(&self) -> Option<String> {
    match self {
      MalType::Symbol(s) => Some(s.to_owned()),
      _ => None,
    }
  }

  pub fn string_value(&self) -> Option<String> {
    match self {
      MalType::String(s) => Some(s.to_owned()),
      _ => None,
    }
  }

  pub fn function_value(&self) -> Option<MalFunc> {
    match self {
      MalType::Function(func) => Some(func.to_owned()),
      _ => None,
    }
  }

  pub fn to_bool(val: bool) -> MalType {
    if val {
      MalType::True
    } else {
      MalType::False
    }
  }
}

pub type CoreFunction = fn(&mut Vec<MalType>, Option<Env>) -> MalResult;

#[derive(Clone)]
pub struct MalFunc {
  pub func: CoreFunction,
  pub env: Option<Env>,
}

#[derive(Clone)]
pub struct MalLambda {
  pub env: Env,
  pub args: Vec<MalType>,
  pub body: Vec<MalType>,
}

impl fmt::Debug for MalFunc {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "#<function>")
  }
}

impl fmt::Debug for MalLambda {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "#<function>")
  }
}

pub type MalResult = Result<MalType, MalError>;

#[derive(Debug, Clone)]
pub enum MalErrorReason {
  Unknown,
  UnexpectedEOF,
  UnexpectedEndOfString,
  SymbolNotFound(String),
  NotAFunction,
  NotANumber,
  WrongArguments(String),
  BlankLine,
  Generic(String),
}

impl fmt::Display for MalErrorReason {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let reason = match self {
      MalErrorReason::Unknown => "Unknown".to_string(),
      MalErrorReason::UnexpectedEOF => "Unexpected EOF".to_string(),
      MalErrorReason::UnexpectedEndOfString => {
        "Unexpected end of string. Possibly unbalanced quotes".to_string()
      }
      MalErrorReason::SymbolNotFound(sym) => format!("Symbol '{}' not found", sym),
      MalErrorReason::NotAFunction => "Expected function".to_string(),
      MalErrorReason::NotANumber => "Expected number".to_string(),
      MalErrorReason::WrongArguments(reason) => format!("Wrong arguments - {}", reason),
      MalErrorReason::BlankLine => "".to_string(),
      MalErrorReason::Generic(reason) => reason.to_string(),
    };
    write!(f, "{}", reason)
  }
}

#[derive(Debug, Clone)]
pub struct MalError {
  reason: MalErrorReason,
}

impl MalError {
  pub fn unexpected_eof() -> MalError {
    MalError {
      reason: MalErrorReason::UnexpectedEOF,
    }
  }

  pub fn unknown() -> MalError {
    MalError {
      reason: MalErrorReason::Unknown,
    }
  }

  pub fn unexpected_end_of_string() -> MalError {
    MalError {
      reason: MalErrorReason::UnexpectedEndOfString,
    }
  }

  pub fn symbol_not_found(sym: &str) -> MalError {
    MalError {
      reason: MalErrorReason::SymbolNotFound(sym.to_string()),
    }
  }

  pub fn not_a_function() -> MalError {
    MalError {
      reason: MalErrorReason::NotAFunction,
    }
  }

  pub fn not_a_number() -> MalError {
    MalError {
      reason: MalErrorReason::NotANumber,
    }
  }

  pub fn wrong_arguments(reason: &str) -> MalError {
    MalError {
      reason: MalErrorReason::WrongArguments(reason.to_string()),
    }
  }

  pub fn blank_line() -> MalError {
    MalError {
      reason: MalErrorReason::BlankLine,
    }
  }

  pub fn generic(reason: &str) -> MalError {
    MalError {
      reason: MalErrorReason::Generic(reason.to_string()),
    }
  }

  pub fn reason(&self) -> &MalErrorReason {
    &self.reason
  }
}

impl fmt::Display for MalError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Error: {}", self.reason)
  }
}

impl error::Error for MalError {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    None
  }
}
