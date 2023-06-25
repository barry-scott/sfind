use std::path::PathBuf;

#[derive(Debug)]
pub struct CommandOptions {
    pub progname:           String,
    pub debug:              bool,
    pub save_config:        bool,
    pub usage:              bool,
    pub find_iname:         bool,
    pub grep_ignore_case:   bool,
    pub lines_after:        Option<u32>,
    pub lines_before:       Option<u32>,
    pub depth:              Option<u32>,
    pub file_contains:      Vec<String>,
    pub folders:            Vec<PathBuf>,
    pub files:              Vec<PathBuf>,
}

impl CommandOptions {
    pub fn new(args: &[String]) -> Result<CommandOptions, String> {
        let mut iargs = args.iter();

        let progname = match iargs.next() {
            Some(arg) => arg.clone(),
            None => {
                return Err(String::from("missing progname in command arguments"));
            }
        };

        let mut opt = CommandOptions {
            progname:           progname,
            debug:              false,
            save_config:        false,
            usage:              false,
            find_iname:         false,
            grep_ignore_case:   false,
            lines_after:        Option::None,
            lines_before:       Option::None,
            depth:              Option::None,
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
                else if "-save-config" == arg {
                    opt.save_config = true;
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
                            return Err(String::from("missing argument to -contains"));
                        }
                    };
                    opt.file_contains.push(contains.clone())
                }
                else if "-after".starts_with( arg ) && arg.len() >= 2 {
                    match Self::parse_integer_arg(iargs.next(), "-after") {
                        Ok(value) => {
                            opt.lines_after = Some(value);
                        }
                        Err(err) => {
                            return Err(err);
                        }
                    };
                }
                else if "-before".starts_with( arg ) && arg.len() >= 2 {
                    match Self::parse_integer_arg(iargs.next(), "-before") {
                        Ok(value) => {
                            opt.lines_before = Some(value);
                        }
                        Err(err) => {
                            return Err(err);
                        }
                    };
                }
                else if "-depth".starts_with( arg ) && arg.len() >= 2 {
                    match Self::parse_integer_arg(iargs.next(), "-depth") {
                        Ok(value) => {
                            opt.depth = Some(value);
                        }
                        Err(err) => {
                            return Err(err);
                        }
                    };
                }
                // look for -<int>
                else if match arg[1..].parse::<u32>() {
                    Ok(value) => {
                        opt.depth = Some(value);
                        true
                    }
                    Err(_) => false
                } {}
                else {
                    return Err(format!("Unknown options \"{arg}\""));
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

    fn parse_integer_arg(arg: Option<&String>, opt_name: &str) -> Result<u32, String> {
        match arg {
            Some(value) => {
                match value.parse::<u32>() {
                    Ok(value) => Ok(value),
                    Err(err) => Err(format!("argument to {opt_name} must be an integer - {err}"))
                }
            }
            None => Err(format!("expecting <int> argument to {opt_name}"))
        }
    }

    pub fn usage(&self) -> String {
        format!("Usage: {0}
    -help               - print this help
    -contains (-c)      - grep for string in found files
    -after <int> (-a)   - some <int> lines after match
    -before <int> (-b)  - some <int> lines before match
    -ignore-case (-i)   - ignore case when greping
    -iname (-in)        - ignore case of filenames
    -save-config        - write the default config
                          into {1}
    -debug              - print the find command line
    -depth <int> (-d)   - limit find to a max depth of <int>
    -<int>              - limit find to a max depth of <int>
", self.progname, "TBD")
    }
}
