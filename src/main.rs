use std::process::ExitCode;
use std::env;

use sfind;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    let cmd_opt = match sfind::CommandOptions::new(&args) {
        Ok(opt) => opt,
        Err(error) => {
            println!("Error: {error}");
            return ExitCode::from(1);
        }
    };

    match sfind::run(cmd_opt) {
        Ok(_) => {
            return ExitCode::SUCCESS;
        }
        Err(error) => {
            println!("Error: {error}");
            return ExitCode::from(1);
        }
    }
}

