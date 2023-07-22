use anyhow::Result;

pub mod find_files;
pub use find_files::FindFiles;

pub mod grep_in_file;
pub use grep_in_file::{GrepPatterns, GrepInFile};

pub mod command_options;
pub use command_options::CommandOptions as CommandOptions;

pub mod config_json;
pub use config_json::AppConfig as AppConfig;

pub fn run(opt: CommandOptions, cfg: AppConfig) -> Result<()> {
    if opt.fixed_strings.len() == 0 && opt.regex_patterns.len() == 0 {
        // just print the files that are found
        for path in FindFiles::new(&opt, &cfg.config) {
            println!("{}", path.display());
        }
    } else {
        let patterns = GrepPatterns::new(&opt)?;

        // search inside each found file
        for path in FindFiles::new(&opt, &cfg.config) {
            if opt.debug {
                println!("grep_in_file {}", path.display());
            }
            let mut grep_in_file = GrepInFile::new(&opt, &path, &patterns);
            match grep_in_file.search() {
                Err(e) => {
                    println!("error {}", e);
                },
                Ok(_) => { },
            }
        }
    }

    Ok(())
}
