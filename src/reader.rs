extern crate regex;

use regex::Regex;

use crate::types::*;

const TOKEN_PATTERN: &str =
  r#"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"|;.*|[^\s\[\]{}('"`,;)]*)"#;
const NUMBER_PATTERN: &str = r#"^\-?[\d\.]+$"#;

pub fn read_str(input: String) -> MalResult {
  let mut reader = Reader::new(tokenize(input));
  read_form(&mut reader)
}

pub struct Reader {
  tokens: Vec<String>,
  position: usize,
}

impl Reader {
  pub fn new(tokens: Vec<String>) -> Reader {
    Reader {
      tokens: tokens,
      position: 0,
    }
  }

  // Return token at current position and increments position
  pub fn next(&mut self) -> Option<String> {
    let current = self.peek();
    if self.position < self.tokens.len() {
      self.position += 1;
    }
    current
  }

  // Return current token
  pub fn peek(&self) -> Option<String> {
    if let Some(token) = self.tokens.get(self.position) {
      Some(token.to_string())
    } else {
      None
    }
  }
}

fn tokenize(input: String) -> Vec<String> {
  let pattern = Regex::new(TOKEN_PATTERN).unwrap();
  let mut tokens = vec![];
  for capture in pattern.captures_iter(&input) {
    if capture[1].starts_with(";") {
      continue;
    }
    tokens.push(capture[1].to_string());
  }
  tokens
}

fn read_form(reader: &mut Reader) -> MalResult {
  if let Some(token) = reader.peek() {
    // println!("Read Form token: {}", token);
    let mut chars = token.chars();
    if let Some(c) = chars.next() {
      match c {
        '(' => read_list(reader),
        '[' => read_vector(reader),
        '{' => read_hashmap(reader),
        '"' => read_string(reader),
        ':' => read_keyword(reader),
        '^' => read_with_meta(reader),
        '\'' => read_quote(reader, "quote"),
        '`' => read_quote(reader, "quasiquote"),
        '@' => read_quote(reader, "deref"),
        '~' => {
          if let Some('@') = chars.next() {
            read_quote(reader, "splice-unquote")
          } else {
            read_quote(reader, "unquote")
          }
        }
        ';' => {
          reader.next();
          return Err(MalError::blank_line());
        }
        _ => read_atom(reader),
      }
    } else {
      Err(MalError::blank_line())
    }
  } else {
    Err(MalError::blank_line())
  }
}

fn read_atom(reader: &mut Reader) -> MalResult {
  let token = reader.next().unwrap();
  let pattern = Regex::new(NUMBER_PATTERN).unwrap();
  let value = if pattern.is_match(&token) {
    MalType::Number(token.parse::<i64>().unwrap_or(0))
  } else {
    match token.as_ref() {
      "nil" => MalType::Nil,
      "true" => MalType::True,
      "false" => MalType::False,
      _ => MalType::Symbol(token),
    }
  };
  Ok(value)
}

fn read_inner_list(reader: &mut Reader, end: char) -> Result<Vec<MalType>, MalError> {
  reader.next(); // Consume opening
  let mut list = vec![];
  loop {
    let token = match reader.peek() {
      Some(t) => t,
      None => return Err(MalError::unexpected_eof()),
    };
    if let Some(c) = token.chars().next() {
      if c == end {
        break;
      } else {
        list.push(read_form(reader)?);
      }
    } else {
      return Err(MalError::unexpected_eof());
    }
  }
  reader.next(); // consume closing
  Ok(list)
}

fn read_list(reader: &mut Reader) -> MalResult {
  let list = read_inner_list(reader, ')')?;
  Ok(MalType::List(list))
}

fn read_vector(reader: &mut Reader) -> MalResult {
  let list = read_inner_list(reader, ']')?;
  Ok(MalType::Vector(list))
}

fn read_hashmap(reader: &mut Reader) -> MalResult {
  let list = read_inner_list(reader, '}')?;
  Ok(MalType::HashMap(list))
}

fn read_string(reader: &mut Reader) -> MalResult {
  let token = reader.next().unwrap();
  let mut chars = token.chars();
  if chars.next().unwrap() != '"' {
    return Err(MalError::unknown());
  }
  let mut out = String::new();
  loop {
    match chars.next() {
      Some('"') => break,
      Some('\\') => out.push(unescape_char(chars.next())?),
      Some(c) => out.push(c),
      None => return Err(MalError::unexpected_end_of_string()),
    }
  }
  Ok(MalType::String(out))
}

fn unescape_char(input: Option<char>) -> Result<char, MalError> {
  match input {
    Some('n') => Ok('\n'),
    Some(c) => Ok(c),
    None => Err(MalError::unexpected_eof()),
  }
}

fn read_keyword(reader: &mut Reader) -> MalResult {
  let token = reader.next().unwrap();
  Ok(MalType::Keyword(token[1..].to_string()))
}

fn read_quote(reader: &mut Reader, label: &str) -> MalResult {
  reader.next().unwrap(); // Consume quote character
  let list = vec![MalType::Symbol(label.to_string()), read_form(reader)?];
  Ok(MalType::List(list))
}

fn read_with_meta(reader: &mut Reader) -> MalResult {
  reader.next().unwrap(); // Consume meta character
  let metadata = read_form(reader)?;
  let value = read_form(reader)?;
  let list = vec![MalType::Symbol("with-meta".to_string()), value, metadata];
  Ok(MalType::List(list))
}
