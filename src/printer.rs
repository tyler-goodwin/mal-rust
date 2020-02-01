extern crate regex;

use regex::Regex;

use crate::types::*;

pub fn print_str(input: &MalType, print_readably: bool) -> String {
  match input {
    MalType::Nil => String::from("nil"),
    MalType::False => String::from("false"),
    MalType::True => String::from("true"),
    MalType::Number(num) => num.to_string(),
    MalType::Symbol(sym) => sym.to_string(),
    MalType::String(s) => {
      if print_readably {
        print_string(s)
      } else {
        s.to_owned()
      }
    }
    MalType::Keyword(s) => String::from(":") + s,
    MalType::List(list) => print_list_like(list, "(", ")", print_readably),
    MalType::Vector(list) => print_list_like(list, "[", "]", print_readably),
    MalType::HashMap(list) => print_list_like(list, "{", "}", print_readably),
    MalType::Function(_) => String::from("#<function>"),
    MalType::Lambda(_) => String::from("#<lambda>"), // TODO: Change back to function
  }
}

fn print_list_like(list: &Vec<MalType>, start: &str, end: &str, print_readably: bool) -> String {
  let mut output = String::from(start);
  output += &list
    .iter()
    .map(|val| print_str(val, print_readably))
    .collect::<Vec<String>>()
    .join(" ");
  output += end;
  output
}

const ESCAPED_QUOTES_PATTERN: &str = r#"\\'"#;

pub fn print_string(input: &str) -> String {
  let output = format!("{:?}", input);
  let pattern = Regex::new(ESCAPED_QUOTES_PATTERN).unwrap();
  pattern.replace_all(&output, "'").into_owned()
}
