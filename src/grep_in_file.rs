use std::collections::VecDeque;
use std::path::PathBuf;

use anyhow::{Result, anyhow};
use regex::{Regex,RegexBuilder};

pub use crate::command_options::CommandOptions as CommandOptions;

pub struct GrepPatterns {
    pub patterns:        Vec<Regex>,
}

pub struct GrepInFile<'caller> {
    patterns:           &'caller GrepPatterns,
    file_path:          &'caller PathBuf,
    num_before:         u32,
    before_lines:       VecDeque<String>,
    line_number:        u64,
    matched_line:       String,
    num_after:          u32,
    after_lines:        VecDeque<String>,
}

impl GrepPatterns {
    pub fn new(opt: &CommandOptions) -> Result<GrepPatterns> {
        let mut gp = GrepPatterns {
            patterns:        vec!(),
        };

        for fixed in &opt.fixed_strings {
            match RegexBuilder::new(&GrepPatterns::quote_regex(&fixed)).case_insensitive(opt.grep_ignore_case).build() {
                Ok(regex) => {
                    gp.patterns.push(regex);
                },
                Err(_) => {
                    return Err(anyhow!("failed to compile regex for {}", fixed))
                }
            }
        }

        for pattern in &opt.regex_patterns {
            match RegexBuilder::new(&pattern).case_insensitive(opt.grep_ignore_case).build() {
                Ok(regex) => {
                    gp.patterns.push(regex);
                },
                Err(_) => {
                    return Err(anyhow!("failed to compile regex for {}", pattern))
                }
            }
        }

        Ok(gp)
    }

    fn quote_regex(text: &String) -> String {
        let mut regex_pattern = String::new();

        for ch in text.chars() {
            match ch {
                '.' | '+' | '*' | '?' | '\\' | '#' | '^' | '$' |
                '(' | ')' | '|' | '[' | ']' | '{' | '}' => {
                    regex_pattern.push('\\');
                    regex_pattern.push(ch);
                }
                _ => {
                    regex_pattern.push(ch)
                }
            }
        }

        regex_pattern
    }
}

impl <'caller> GrepInFile<'caller> {
    pub fn new(opt: &'caller CommandOptions, file_path: &'caller PathBuf, patterns: &'caller GrepPatterns) -> GrepInFile<'caller> {
        let gif = GrepInFile {
            patterns:           patterns,
            file_path:          file_path,
            num_before:         match opt.grep_lines_before { Some(n) => n, None => 0 },
            before_lines:       VecDeque::new(),
            line_number:        0,
            matched_line:       String::new(),
            num_after:          match opt.grep_lines_after { Some(n) => n, None => 0 },
            after_lines:        VecDeque::new(),
        };

        gif
    }

    pub fn search(&mut self) -> Result<()> {
        return Ok(());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quote_regex() {
        assert_eq!(GrepPatterns::quote_regex("fixed"), "fixed");
        assert_eq!(GrepPatterns::quote_regex("file.type"), "file\\.type");
        assert_eq!(GrepPatterns::quote_regex("*.type"), ".*\\.type");
    }
}
