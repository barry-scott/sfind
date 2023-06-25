use std::process::{Command, Stdio};
use std::io::{BufRead, BufReader, BufWriter};
use std::io::{self, Write};

pub mod command_options;
pub use command_options::CommandOptions as CommandOptions;

pub mod config_json;
pub use config_json::ConfigJson as ConfigJson;
pub use config_json::AppConfig as AppConfig;

pub fn run(opt: CommandOptions, cfg: AppConfig) -> Result<(), String> {
    if opt.usage {
        println!("{}", opt.usage(&cfg));
        return Ok(());
    }

    if opt.save_default_config {
        match cfg.save_default_config() {
            Ok(_) => return Ok(()),
            Err(e) => return Err(e.to_string())
        }
    }

    let mut cmd = Command::new("/usr/bin/find");
    build_command(&mut cmd, &opt, &cfg.config);

    if opt.debug {
        let mut stdout = BufWriter::new(io::stdout());

        let _ = stdout.write_all(cmd.get_program().to_str().unwrap().as_bytes());
        for arg in cmd.get_args() {
            let _ = stdout.write_all(" ".as_bytes());
            let _ = stdout.write_all(arg.to_str().unwrap().as_bytes());
        };
        let _ = stdout.write("\n".as_bytes());
        let _ = stdout.flush();
    };

    let proc = match cmd.spawn() {
        Ok(child) => child,
        Err(err) => return Err(err.to_string())
    };

    let mut stdout = BufReader::new(proc.stdout.unwrap());

    let mut line = String::new();
    while match stdout.read_line(&mut line) {
        Ok(0) => false,
        Ok(_) => true,
        Err(_) => false
    } {
        print_line(&line);
        line.clear();
    }

    Ok(())
}

fn print_line(line: &str) {
    let parts: Vec<&str> = line.splitn(3, "\x1b[99m:\x1b[m").collect();

    let mut stdout = BufWriter::new(io::stdout());
    if parts.len() == 3 {
        let filename = parts[0];
        let linenum = parts[1];
        let matched = parts[2];

        let prefix_len = filename.len() + 1 + linenum.len() + 1 + 1;
        let pad = 4 - (prefix_len%4);
        let pad: &str = &"    "[..pad];
        let line = format!("\x1b[35m{filename}\x1b[m:\x1b[32m{linenum}\x1b[m: {pad}{matched}");
        let _ = stdout.write(line.as_bytes());
    } else {
        let _ = stdout.write(line.as_bytes());
    }
    let _ = stdout.flush();
}

fn build_command(cmd: &mut Command, opt: &CommandOptions, cfg: &ConfigJson) {
    let cmd = cmd.stdout(Stdio::piped());
    for folder in opt.folders.iter() {
        let _ = cmd.arg(folder);
    }
    match opt.find_depth {
        Some(depth) => {
            let _ = cmd.arg("-maxdepth").arg(depth.to_string());
        },
        None => ()
    };
    let _ = cmd.arg("!").arg("(").arg("(");
    let mut sep = false;
    for folder in cfg.folders_to_prune.iter() {
        if sep {
            let _ = cmd.arg("-o");
        }
        let _ = cmd.arg("-path").arg(format!("*/{folder}"));
        sep = true;
    };
    let _ = cmd.arg(")").arg("-prune").arg(")");
    if opt.files.len() > 0 {
        let mut sep = false;
        let _ = cmd.arg("(");
        for file in opt.files.iter() {
            if sep {
                let _ = cmd.arg("-o");
            }
            if opt.find_iname {
                let _ = cmd.arg("-iname");
            } else {
                let _ = cmd.arg("-name");
            }
            let _ = cmd.arg(file);
            sep = true;
        };
        let _ = cmd.arg(")");
    };

    for file in cfg.files_to_prune.iter() {
        let _ = cmd.arg("!").arg("-name").arg(file);
    };

    if opt.file_contains.len() > 0 {
        // turn kill to end of line in the output with ne
        // fn=:ln=:se=99 marks the : with \e[99m:\e[m
        let _ = cmd.env("GREP_COLORS", "ne:fn=:ln=:se=99");

        let _ = cmd.arg("-type").arg("f").arg("-exec").arg("grep");
        if opt.grep_ignore_case {
            let _ = cmd.arg("--ignore-case");
        };

        match opt.grep_lines_after {
            Some(lines) => {
                let _ = cmd.arg(format!("--after-context={}", lines));
            },
            None => {}
        };
        match opt.grep_lines_before {
            Some(lines) => {
                let _ = cmd.arg(format!("--before-context={}", lines));
            },
            None => {}
        };

        let _ = cmd.arg("--color=always").arg("--with-filename").arg("--line-number");
        for pattern in opt.file_contains.iter() {
            let _ = cmd.arg("-e").arg(pattern);
        };
        let _ = cmd.arg("{}").arg("+");
    };
}
