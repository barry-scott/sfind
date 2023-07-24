use std::env;
use std::process::ExitCode;



fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    let cmd_opt = match sfind::CommandOptions::new(&args) {
        Ok(opt) => opt,
        Err(error) => {
            println!("Error: {error}");
            return ExitCode::from(1);
        }
    };

    if cmd_opt.debug {
        dbg!(&cmd_opt);
    };

    let cfg = match sfind::AppConfig::new("org.barrys-emacs.smart-find") {
        Ok(cfg) => cfg,
        Err(error) => {
            println!("Error: {error}");
            return ExitCode::from(1);
        }
    };

    if cmd_opt.save_default_config {
        match cfg.save_default_config() {
            Err(e) => {
                println!("failed to save default config - {}", e);
                return ExitCode::from(1);
            }
            Ok(()) => {
                return ExitCode::SUCCESS;
            }
        }
    }
    if cmd_opt.show_config {
        cfg.show_config();
        return ExitCode::SUCCESS;
    }

    if cmd_opt.debug {
        dbg!(&cfg);
    };

    match sfind::run(cmd_opt, cfg) {
        Ok(_) => {
            ExitCode::SUCCESS
        }
        Err(error) => {
            println!("Error: {error}");
            ExitCode::from(1)
        }
    }
}
