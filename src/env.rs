use std::collections::HashMap;
use std::rc::Rc;

use crate::types::*;

#[derive(Clone)]
pub struct Env {
  data: HashMap<String, MalType>,
  parent: Option<Rc<Env>>,
}

impl Env {
  pub fn new(parent: Option<Env>) -> Env {
    let parent = match parent {
      Some(env) => Some(Rc::from(env)),
      None => None,
    };
    Env {
      data: HashMap::new(),
      parent: parent,
    }
  }

  pub fn set(&mut self, key: &str, value: MalType) {
    self.data.insert(key.to_string(), value);
  }

  pub fn find(&self, key: &str) -> Option<Env> {
    if self.data.contains_key(key) {
      Some(self.clone())
    } else {
      match &self.parent {
        Some(env) => env.find(key),
        None => None,
      }
    }
  }

  pub fn get(&self, key: &str) -> MalResult {
    match self.find(key) {
      Some(env) => Ok(env.data.get(key).unwrap().to_owned()),
      None => Err(MalError::symbol_not_found(key.to_string())),
    }
  }
}
