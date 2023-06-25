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

    dbg!(&cmd_opt);

    let cfg = match sfind::ConfigJson::new() {
        Ok(cfg) => cfg,
        Err(error) => {
            println!("Error: {error}");
            return ExitCode::from(1);
        }
    };

    dbg!(&cfg);

    match sfind::run(cmd_opt, cfg) {
        Ok(_) => {
            return ExitCode::SUCCESS;
        }
        Err(error) => {
            println!("Error: {error}");
            return ExitCode::from(1);
        }
    }
}

