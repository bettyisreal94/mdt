use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, BufRead};
use std::process::ExitCode;

mod parser;

const DEFAULT_INPUT: &str = "stdin";
const DEFAULT_OUTPUT: &str = "stdout";

fn usage() -> String {
    "Usage: mdt [OPTIONS]\n\
     \n\
     Options:\n\
     -i <FILE>  Input file (default: stdin)\n\
     -o <FILE>  Output file (default: stdout)\n\
     -h         Show this help message".to_string()
}

struct Args {
    input: String,
    output: String,
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Args, String> {
    let mut input = DEFAULT_INPUT.to_string();
    let mut output = DEFAULT_OUTPUT.to_string();

    let mut iter = args;
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-i" => {
                input = iter
                    .next()
                    .ok_or_else(|| "-i requires a value".to_string())?;
            }
            "-o" => {
                output = iter
                    .next()
                    .ok_or_else(|| "-o requires a value".to_string())?;
            }
            "-h" | "--help" => {
                println!("{}", usage());
                std::process::exit(0);
            }
            other => {
                return Err(format!("unknown argument: {other}"));
            }
        }
    }

    Ok(Args { input, output })
}

fn main() -> ExitCode {
    let args = match parse_args(env::args().skip(1)) {
        Ok(args) => args,
        Err(err) => {
            eprintln!("{err}\n\n{}", usage());
            return ExitCode::FAILURE;
        }
    };

    // Read input from file or stdin
    let table_str = match &args.input {
        DEFAULT_INPUT => io::stdin()
            .lock()
            .lines()
            .collect::<Result<Vec<String>, _>>()
            .map_err(|e| format!("Failed to read stdin: {}", e)),
        path => fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file '{}': {}", path, e))?,
    };

    // Parse the markdown table
    let result = parser::parse(&table_str);

    match result {
        Ok(table) => {
            // Output the parsed table as markdown
            if &args.output == DEFAULT_OUTPUT {
                io::stdout()
                    .write_all(format!("{}", table).as_bytes())
                    .unwrap();
            } else {
                fs::write(&args.output, format!("{}", table))
                    .expect("Failed to write output");
            }
        }
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }

    ExitCode::SUCCESS
}
