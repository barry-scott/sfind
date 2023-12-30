use std::env;
use std::process::ExitCode;
use clap;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();

    let cmd_opt = match sfind::CommandOptions::new(&args) {
        Ok(opt) => opt,
        Err(error) => {
            // there must be a better way to see if the text is from clap...
            use clap::error::ErrorKind;
            match error.downcast_ref::<clap::error::Error>() {
                Some(error) if matches!(error.kind(),
                    ErrorKind::DisplayHelp | ErrorKind::UnknownArgument |
                    ErrorKind::TooFewValues | ErrorKind::InvalidValue) => {
                        eprintln!("{error}")
                    }
                Some(error) => {
                        eprintln!("Error: {error}\nkind: {:?}", error.kind())
                    }
                None => eprintln!("Error: {error}")
                }
            return ExitCode::from(1);
        }
    };

    if cmd_opt.debug {
        dbg!(&cmd_opt);
    };

    let cfg = match sfind::AppConfig::new("org.barrys-emacs.smart-find") {
        Ok(cfg) => cfg,
        Err(error) => {
            eprintln!("Error: {error}");
            return ExitCode::from(1);
        }
    };

    if cmd_opt.save_default_config {
        match cfg.save_default_config() {
            Err(e) => {
                eprintln!("Error: Failed to save default config - {}", e);
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
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::from(1)
        }
    }
}
