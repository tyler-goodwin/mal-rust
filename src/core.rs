use crate::types::*;

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

pub fn plus(args: &mut Vec<MalType>) -> MalResult {
  let result = to_numbers(args)?.iter().fold(0, |acc, x| acc + x);
  Ok(MalType::Number(result))
}

pub fn minus(args: &mut Vec<MalType>) -> MalResult {
  let mut args = to_numbers(args)?;
  let mut result = args.remove(0);
  for i in args {
    result -= i;
  }
  Ok(MalType::Number(result))
}

pub fn multiply(args: &mut Vec<MalType>) -> MalResult {
  let result = to_numbers(args)?.iter().fold(1, |acc, x| acc * x);
  Ok(MalType::Number(result))
}

pub fn divide(args: &mut Vec<MalType>) -> MalResult {
  let mut args = to_numbers(args)?;
  let mut result = args.remove(0);
  for i in args {
    result /= i;
  }
  Ok(MalType::Number(result))
}
