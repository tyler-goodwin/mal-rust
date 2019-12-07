use std::io::{self, Write};

enum ReadResult {
    InputRecv(String),
    Done(()),
}

enum RepResult {
    Continue,
    Done,
}

fn read() -> Result<ReadResult, io::Error> {
    let mut input = String::new();
    print!("user> ");
    io::stdout().flush()?;

    let bytes = io::stdin().read_line(&mut input)?;
    if bytes != 0 {
        Ok(ReadResult::InputRecv(input))
    } else {
        Ok(ReadResult::Done(()))
    }
}

fn eval(input: String) -> Result<String, io::Error> {
    Ok(input)
}

fn print(output: String) {
    print!("{}", output);
    io::stdout().flush().unwrap();
}

fn rep() -> Result<RepResult, io::Error> {
    match read()? {
        ReadResult::InputRecv(input) => {
            print(eval(input)?);
            Ok(RepResult::Continue)
        }
        ReadResult::Done(_) => Ok(RepResult::Done),
    }
}

fn main() {
    loop {
        match rep() {
            Ok(RepResult::Continue) => (),
            Ok(RepResult::Done) => break,
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    println!();
}
