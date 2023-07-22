use std::path::PathBuf;
use anyhow::{Result, anyhow};
use clap::{Parser};

#[derive(Debug)]
#[derive(Parser)]
#[command(name = "sfind")]
#[command(version = "1.0")]
#[command(about = "smart find", long_about=r#"Search for all <filename>'s in all <dir>'s
    If -contains is present grep for all <patterns> in the found files."#)]
struct Cli {
    #[arg(long)]
    pub debug:                      bool,

    #[arg(long, help="write the default config info <TBD>")]
    pub save_default_config:        bool,

    #[arg(short='S', long="asis", help="find file names matching case")]
    pub case_sensitive_filenames:   bool,

    #[arg(short='s', long="sensitive", help="match regex case sensitively")]
    pub case_sensitive_contents:    bool,

    #[arg(short, long, value_name="LINES", help="lines to show after match")]
    pub after:                      Option<u32>,

    #[arg(short, long, value_name="LINES", help="lines to show before match")]
    pub before:                     Option<u32>,

    #[arg(short, long, help="number of folder levels to search")]
    pub depth:                      Option<u32>,

    #[arg(short, long, value_name="REGEX", help="regex pattern to find")]
    pub regex:                      Vec<String>,

    #[arg(short, long, value_name="STR", help="fixed string to find")]
    pub fixed:                      Vec<String>,

    #[arg(value_name="PATH", help="Files and Folders to find")]
    pub positional:                 Vec<PathBuf>,
}

#[derive(Debug)]
pub struct CommandOptions {
    pub progname:               String,
    pub debug:                  bool,
    pub save_default_config:    bool,
    pub find_iname:             bool,
    pub grep_ignore_case:       bool,
    pub grep_lines_after:       Option<u32>,
    pub grep_lines_before:      Option<u32>,
    pub find_depth:             Option<u32>,
    pub regex_patterns:         Vec<String>,
    pub fixed_strings:          Vec<String>,
    pub folders:                Vec<PathBuf>,
    pub files:                  Vec<String>,
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
            progname:               progname,
            debug:                  cli.debug,
            save_default_config:    cli.save_default_config,
            find_iname:             !cli.case_sensitive_filenames,
            grep_ignore_case:       !cli.case_sensitive_contents,
            grep_lines_after:       cli.after,
            grep_lines_before:      cli.before,
            find_depth:             cli.depth,
            regex_patterns:         cli.regex,
            fixed_strings:          cli.fixed,
            folders:                vec![],
            files:                  vec![],
        };

        for path in cli.positional {
            if path.is_dir() {
                opt.folders.push(path);
            } else {
                match path.to_str() {
                    Some(file) => {
                        opt.files.push(file.to_string());
                    },
                    None => {
                        println!("filename is not utf-8 {}", path.display());
                    }
               };
            }
        }

        if opt.folders.len() == 0 {
            opt.folders.push(PathBuf::from("."));
        }

        Ok(opt)
    }
}
