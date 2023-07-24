use anyhow::Result;

pub mod find_files;
pub use find_files::FindFiles;

pub mod grep_in_file;
pub use grep_in_file::{GrepInFile, GrepPatterns};

pub mod command_options;
pub use command_options::CommandOptions;

pub mod config_json;
pub use config_json::AppConfig;

pub fn run(opt: CommandOptions, cfg: AppConfig) -> Result<()> {
    if opt.fixed_strings.is_empty() && opt.regex_patterns.is_empty() {
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
            if let Err(e) = grep_in_file.search() {
                println!("error {}", e);
            }
        }
    }

    Ok(())
}
