use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::types::*;

#[derive(Clone)]
struct EnvType {
  pub data: HashMap<String, MalType>,
  pub parent: Option<Env>,
}

#[derive(Clone)]
pub struct Env(Rc<RefCell<EnvType>>);

impl Env {
  pub fn new(parent: Option<Env>) -> Env {
    Env(Rc::new(RefCell::new(EnvType {
      data: HashMap::new(),
      parent: parent.map(|p| p.clone()),
    })))
  }

  pub fn new_with_bindings(
    parent: Option<Env>,
    binds: Vec<String>,
    mut exprs: Vec<MalType>,
  ) -> Env {
    let mut env = Env::new(parent);
    let mut is_more = false;
    for bind in binds {
      if bind == "&" {
        is_more = true;
      } else if is_more {
        env.set(&bind, MalType::List(exprs));
        break;
      } else if exprs.len() > 0 {
        env.set(&bind, exprs.remove(0));
      }
    }
    env
  }

  pub fn set(&mut self, key: &str, value: MalType) {
    self.0.borrow_mut().data.insert(key.to_string(), value);
  }

  pub fn find(&self, key: &str) -> Option<Env> {
    if self.0.borrow_mut().data.contains_key(key) {
      Some(self.clone())
    } else {
      match &self.0.borrow().parent {
        Some(env) => env.find(key),
        None => None,
      }
    }
  }

  pub fn get(&self, key: &str) -> MalResult {
    match self.find(key) {
      Some(env) => Ok(env.0.borrow().data.get(key).unwrap().to_owned()),
      None => Err(MalError::symbol_not_found(key)),
    }
  }
}
