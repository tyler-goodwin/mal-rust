use std::{error, fmt};

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
}

pub type MalResult = Result<MalType, MalError>;

#[derive(Debug, Clone, Copy)]
pub enum MalErrorReason {
  Unknown,
  UnexpectedEOF,
  UnexpectedEndOfString,
  BlankLine,
}

impl fmt::Display for MalErrorReason {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let reason = match self {
      MalErrorReason::Unknown => "Unkown",
      MalErrorReason::UnexpectedEOF => "Unexpected EOF",
      MalErrorReason::UnexpectedEndOfString => {
        "Unexpected end of string. Possibly unbalanced quotes"
      }
      MalErrorReason::BlankLine => "",
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

  pub fn blank_line() -> MalError {
    MalError {
      reason: MalErrorReason::BlankLine,
    }
  }

  pub fn reason(&self) -> MalErrorReason {
    self.reason
  }
}

impl fmt::Display for MalError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Syntax Error - {}", self.reason)
  }
}

impl error::Error for MalError {
  fn source(&self) -> Option<&(dyn error::Error + 'static)> {
    None
  }
}
