#[derive(Debug)]
pub struct CommandOptions {
    pub progname:           String,
    pub debug:              bool,
    pub save_config:        bool,
    pub usage:              bool,
    pub find_iname:         bool,
    pub grep_ignore_case:   bool,
    pub depth:              Option<u32>,
    pub file_contains:      Vec<String>,
    pub files:              Vec<String>,
}

impl CommandOptions {
    pub fn usage(&self) -> String {
        format!("Usage: {0}
    -help               - print this help
    -contains (-c)      - grep for string in found files
    -ignore-case (-i)   - ignore case when greping
    -iname (-in)        - ignore case of filenames
    -save-config        - write the default config
                          into {1}
    -debug              - print the find command line
    -<int>              - limit find to a max depth of <int>
", self.progname, "TBD")
    }

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
            depth:              Option::None,
            file_contains:      vec![],
            files:              vec![],
        };

        let mut looking_for_opts = true;

        while let Some(arg) = iargs.next() {
            println!("Arg: {arg}");

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
                // look for -<int>
                else if match arg[1..].parse::<u32>() {
                    Ok(value) => {
                        opt.depth = Some(value);
                        true
                    }
                    Err(_) => false
                } {
                    true;
                }
                else {
                    return Err(format!("Unknown options \"{arg}\""));
                }
            }
            else {
                // assume its a file for now
                opt.files.push(arg.clone());
            }
        }

        Ok(opt)
    }
}
