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
    let input = fs::read(&filename).map_err(|_| CliError::FailedToOpenFile)?;

    let mut vm = VM::new();
    vm.load_program(input);

    vm.run_program();

    println!("{:?}", vm);

    Ok(())
}

fn main() {
    if let Err(error) = try_main() {
        eprintln!("{:?}", error);
        process::exit(1)
    }
}
