use std::path::PathBuf;
use crate::config_json::AppConfig;
use anyhow::{Result, anyhow};

#[derive(Debug)]
pub struct CommandOptions {
    pub progname:               String,
    pub debug:                  bool,
    pub usage:                  bool,
    pub save_default_config:    bool,
    pub find_iname:             bool,
    pub grep_ignore_case:       bool,
    pub grep_lines_after:       Option<u32>,
    pub grep_lines_before:      Option<u32>,
    pub find_depth:             Option<u32>,
    pub file_contains:          Vec<String>,
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

        let mut opt = CommandOptions {
            progname:           progname,
            debug:              false,
            save_default_config:false,
            usage:              false,
            find_iname:         false,
            grep_ignore_case:   false,
            grep_lines_after:   Option::None,
            grep_lines_before:  Option::None,
            find_depth:         Option::None,
            file_contains:      vec![],
            folders:            vec![],
            files:              vec![],
        };

        let mut looking_for_opts = true;

        while let Some(arg) = iargs.next() {
            if looking_for_opts && arg.starts_with("-") {
                if "--" == arg {
                    looking_for_opts = false;
                }
                else if "-debug" == arg {
                    opt.debug = true;
                }
                else if "-save-default-config" == arg {
                    opt.save_default_config = true;
                }
                else if "-help".starts_with(arg) && arg.len() >= 2 {
                    opt.usage = true;
                }
                else if "--help" == arg {
                    opt.usage = true;
                }
                else if "-iname".starts_with( arg ) && arg.len() >= 3 {
                    opt.find_iname = true
                }
                else if "-ignore-case".starts_with( arg ) && arg.len() >= 2 {
                    opt.grep_ignore_case = true
                }
                else if "-contains".starts_with( arg ) && arg.len() >= 2 {
                    let contains = match iargs.next() {
                        Some(arg) => arg,
                        None => {
                            return Err(anyhow!("missing argument to -contains"));
                        }
                    };
                    opt.file_contains.push(contains.clone())
                }
                else if "-after".starts_with( arg ) && arg.len() >= 2 {
                    opt.grep_lines_after = Some(Self::parse_integer_arg(iargs.next(), "-after")?);
                }
                else if "-before".starts_with( arg ) && arg.len() >= 2 {
                    opt.grep_lines_before = Some(Self::parse_integer_arg(iargs.next(), "-before")?);
                }
                else if "-depth".starts_with( arg ) && arg.len() >= 2 {
                    opt.find_depth = Some(Self::parse_integer_arg(iargs.next(), "-depth")?);
                }
                // look for -<int>
                else if match arg[1..].parse::<u32>() {
                    Ok(value) => {
                        opt.find_depth = Some(value);
                        true
                    }
                    Err(_) => false
                } {}
                else {
                    return Err(anyhow!("Unknown options \"{arg}\""));
                }
            }
            else {
                let path = PathBuf::from(arg);

                if path.is_dir() {
                    opt.folders.push(path);
                } else {
                    opt.files.push(path);
                }
            }
        }

        if opt.folders.len() == 0 {
            opt.folders.push(PathBuf::from("."));
        }

        Ok(opt)
    }

    fn parse_integer_arg(arg: Option<&String>, opt_name: &str) -> Result<u32> {
        match arg {
            Some(value) => {
                match value.parse::<u32>() {
                    Ok(value) => Ok(value),
                    Err(err) => Err(anyhow!("argument to {opt_name} must be an integer - {err}"))
                }
            }
            None => Err(anyhow!("expecting <int> argument to {opt_name}"))
        }
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
