use std::path::PathBuf;
use crate::config_json::AppConfig;
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
    pub regex_pattern:          Vec<String>,
    pub fixed_string:           Vec<String>,
    pub folders:                Vec<PathBuf>,
    pub files:                  Vec<PathBuf>,
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
            regex_pattern:          cli.regex,
            fixed_string:           cli.fixed,
            folders:                vec![],
            files:                  vec![],
        };

        for path in cli.positional {
            if path.is_dir() {
                opt.folders.push(path);
            } else {
                opt.files.push(path);
            }
        }

        if opt.folders.len() == 0 {
            opt.folders.push(PathBuf::from("."));
        }

        Ok(opt)
    }

    pub fn usage(&self, app_config: &AppConfig) -> Result<String> {
        Ok(format!(r#"Usage: {0} [<dir>...] [<filename>...] [options]...
    Search for all <filename>'s in all <dir>'s
    If -contains is present grep for all <patterns> in the found files.

    -help                       - print this help
    -contains <pattern> (-c)    - grep for string in found files
    -after <int> (-a)           - some <int> lines after match
    -before <int> (-b)          - some <int> lines before match
    -ignore-case (-i)           - ignore case when greping
    -iname (-in)                - ignore case of filenames
    -save-default-config        - write the default config
                                  into {1}
    -depth <int> (-d, -<int>)   - limit find to a max depth of <int>
    -debug                      - print debug messages

    The JSON config file allows for pruning filenames and folders.

    Example:
    {{
        "folders_to_prune": [".svn", ".git", ".hg"],
        "files_to_prune":   ["*~"]
    }}

"#, self.progname, app_config.config_file_path()?.display()))
    }
}
