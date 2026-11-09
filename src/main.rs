use std::env;
use std::process::ExitCode;

const DEFAULT_INPUT: &str = "stdin";
const DEFAULT_OUTPUT: &str = "stdout";

fn usage() -> String {
    "Usage: mdt [OPTIONS]\n\
     \n\
     Options:\n\
     \x20  -i <FILE>  Input file (default: stdin)\n\
     \x20  -o <FILE>  Output file (default: stdout)\n\
     \x20  -h         Show this help message".to_string()
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

    println!("input:  {}", args.input);
    println!("output: {}", args.output);

    ExitCode::SUCCESS
}
