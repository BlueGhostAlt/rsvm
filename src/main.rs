#![feature(process_exitcode_placeholder, termination_trait_lib)]

use std::{env, fmt, fs, process};

use rsvm::VM;

use colored::Colorize;

enum CliError {
    NoFileProvided,
    FailedToOpenFile,
}

impl fmt::Debug for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CliError::NoFileProvided => {
                write!(
                    f,
                    "{}{} {}",
                    "[ERROR]".bright_red(),
                    ":".bright_white(),
                    "Please provide a valid file.".cyan()
                )
            }
            CliError::FailedToOpenFile => {
                write!(
                    f,
                    "{}{} {}\n    {}",
                    "[ERROR]".bright_red(),
                    ":".bright_white(),
                    "Failed to open file!".cyan(),
                    "Please make sure the file exists and can be read.".white()
                )
            }
        }
    }
}

fn try_main() -> Result<(), CliError> {
    let args = env::args().collect::<Vec<_>>();

    let filename = args.get(1).ok_or(CliError::NoFileProvided)?;
    let input = fs::read_to_string(&filename).map_err(|_| CliError::FailedToOpenFile)?;

    let vm = VM::new();
    println!("{}\n{}", filename, input);
    Ok(())
}

fn main() {
    match try_main() {
        Err(error) => {
            eprintln!("{:?}", error);
            process::exit(1)
        }
        _ => {}
    }
}
