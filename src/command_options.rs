use anyhow::{anyhow, Result};
use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "sfind")]
#[command(version = env!("CARGO_PKG_VERSION"))]
#[command(
    about = "sfind - smart find files and contents",
    long_about = r#"sfind Search for all filename PATHs in all folder PATHs

If --fixed (-f) or --regex (-r) is present grep for all <patterns>
in the found files."#
)]
struct Cli {
    #[arg(long, help = "write the default settings into the config file")]
    pub save_default_config: bool,

    #[arg(long, help = "show the config settings file")]
    pub show_config: bool,

    #[arg(short = 'S', long = "asis", help = "find file names matching case")]
    pub case_sensitive_filenames: bool,

    #[arg(short = 's', long = "sensitive", help = "match regex case sensitively")]
    pub case_sensitive_contents: bool,

    #[arg(short = 'p', long = "path", help = "match file names anywhere in the full path")]
    pub match_path: bool,

    #[arg(short, long, value_name = "LINES", help = "lines to show after match")]
    pub after: Option<usize>,

    #[arg(short, long, value_name = "LINES", help = "lines to show before match")]
    pub before: Option<usize>,

    #[arg(short, long, help = "number of folder levels to search")]
    pub depth: Option<usize>,

    #[arg(long, help = "report supressed errors")]
    pub errors: bool,

    #[arg(short, long, value_name = "REGEX", help = "regex pattern to find")]
    pub regex: Vec<String>,

    #[arg(short, long, value_name = "STR", help = "fixed string to find")]
    pub fixed: Vec<String>,

    #[arg(value_name = "PATH", help = "Files and Folders to find")]
    pub positional: Vec<PathBuf>,

    #[arg(long, help = "print infomation useful for debugging problems with sfind")]
    pub debug: bool,
}

#[derive(Debug)]
pub struct CommandOptions {
    pub progname: String,
    pub debug: bool,
    pub save_default_config: bool,
    pub show_config: bool,
    pub report_supressed_errors: bool,
    pub find_iname: bool,
    pub find_match_basename: bool,
    pub grep_ignore_case: bool,
    pub grep_lines_after: Option<usize>,
    pub grep_lines_before: Option<usize>,
    pub find_depth: Option<usize>,
    pub regex_patterns: Vec<String>,
    pub fixed_strings: Vec<String>,
    pub folders: Vec<PathBuf>,
    pub files: Vec<String>,
}

impl CommandOptions {
    pub fn new(args: &[String]) -> Result<CommandOptions> {
        let mut iargs = args.iter();

        let progname = match iargs.next() {
            Some(arg) => arg.clone(),
            None => {
                return Err(anyhow!("missing progname in command arguments"));
            }
        };

        let cli = Cli::try_parse()?;
        let mut opt = CommandOptions {
            progname,
            debug: cli.debug,
            save_default_config: cli.save_default_config,
            show_config: cli.show_config,
            report_supressed_errors: cli.errors,
            find_iname: !cli.case_sensitive_filenames,
            find_match_basename: !cli.match_path,
            grep_ignore_case: !cli.case_sensitive_contents,
            grep_lines_after: cli.after,
            grep_lines_before: cli.before,
            find_depth: cli.depth,
            regex_patterns: cli.regex,
            fixed_strings: cli.fixed,
            folders: vec![],
            files: vec![],
        };

        for path in cli.positional {
            if path.is_dir() {
                opt.folders.push(path);
            } else {
                match path.to_str() {
                    Some(file) => {
                        opt.files.push(file.to_string());
                    }
                    None => {
                        eprintln!("filename is not utf-8 {}", path.display());
                    }
                };
            }
        }

        if opt.folders.is_empty() {
            opt.folders.push(PathBuf::from("."));
        }

        Ok(opt)
    }
}
