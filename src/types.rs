use std::{error, fmt};

#[derive(Clone)]
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
}

impl MalType {
  pub fn is_list(&self) -> bool {
    match self {
      MalType::List(_) => true,
      _ => false,
    }
  }

  pub fn list_value(&self) -> Option<Vec<MalType>> {
    match self {
      MalType::List(list) => Some(list.to_owned()),
      _ => None,
    }
  }

  pub fn number_value(&self) -> Option<i64> {
    match self {
      MalType::Number(n) => Some(*n),
      _ => None,
    }
  }
}

#[derive(Clone)]
pub struct MalFunc {
  pub func: fn(&mut Vec<MalType>) -> MalResult,
}

pub type MalResult = Result<MalType, MalError>;

#[derive(Debug, Clone)]
pub enum MalErrorReason {
  Unknown,
  UnexpectedEOF,
  UnexpectedEndOfString,
  SymbolNotFound(String),
  NotAFunction,
  BlankLine,
}

impl fmt::Display for MalErrorReason {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let reason = match self {
      MalErrorReason::Unknown => "Unknown".to_string(),
      MalErrorReason::UnexpectedEOF => "Unexpected EOF".to_string(),
      MalErrorReason::UnexpectedEndOfString => {
        "Unexpected end of string. Possibly unbalanced quotes".to_string()
      }
      MalErrorReason::SymbolNotFound(sym) => format!("Could not find symbol '{}'", sym),
      MalErrorReason::NotAFunction => "Expected function".to_string(),
      MalErrorReason::BlankLine => "".to_string(),
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

  pub fn symbol_not_found(sym: String) -> MalError {
    MalError {
      reason: MalErrorReason::SymbolNotFound(sym),
    }
  }

  pub fn not_a_function() -> MalError {
    MalError {
      reason: MalErrorReason::NotAFunction,
    }
  }

  pub fn blank_line() -> MalError {
    MalError {
      reason: MalErrorReason::BlankLine,
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
