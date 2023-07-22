use std::collections::VecDeque;
use std::path::PathBuf;

use std::fs;
use std::io::{BufRead, BufReader};

use anyhow::{Result, anyhow};
use regex::{Regex, RegexBuilder, Match};

pub use crate::command_options::CommandOptions as CommandOptions;

pub struct GrepPatterns {
    pub patterns:        Vec<Regex>,
}

pub struct GrepInFile<'caller> {
    patterns:           &'caller GrepPatterns,
    file_path:          &'caller PathBuf,
    num_before:         usize,
    before_lines:       VecDeque<String>,
    line_number:        usize,
    num_after:          usize,
}

impl <'caller> GrepPatterns {
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

    pub fn find_match(&'caller self, line: &'caller str) -> Option<Match> {
        for regex in self.patterns.iter() {
            // assuming here that is_match is faster then find
            if regex.is_match(&line) {
                return regex.find(&line);
            }
        }

        None
    }

    fn quote_regex(text: &String) -> String {
        let mut regex_pattern = String::new();

        for ch in text.chars() {
            match ch {
                '.' | '+' | '*' | '?' | '#' | '^' | '$' | '\\' |
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
            num_after:          match opt.grep_lines_after { Some(n) => n, None => 0 },
        };

        gif
    }

    const COLOUR_FILE: &str = "\x1b[35m";
    const COLOUR_LINE: &str = "\x1b[32m";
    const COLOUR_MATCH: &str = "\x1b[1;31m";
    const COLOUR_END: &str = "\x1b[m";

    pub fn search(&mut self) -> Result<()> {
        let file = fs::File::open(self.file_path)?;
        let reader = BufReader::new(file);

        let mut required_after = 0;

        for line_result in reader.lines() {
            match line_result {
                Err(e) => {
                    return Err(anyhow!("Error reading {} - {}", self.file_path.display(), e));
                }
                Ok(line) => {
                    self.line_number += 1;

                    match self.patterns.find_match(&line) {
                        Some(m) => {
                            let mut line_number = self.line_number - self.before_lines.len();

                            loop {
                                match self.before_lines.pop_front() {
                                    Some(line) => {
                                        println!("{}{}{}:{}{}{}- {}",
                                            GrepInFile::COLOUR_FILE, self.file_path.display(), GrepInFile::COLOUR_END,
                                            GrepInFile::COLOUR_LINE, line_number, GrepInFile::COLOUR_END, &line);
                                        line_number += 1;
                                    },
                                    None => break
                                }
                            }
                            println!("{}{}{}:{}{}{}: {}{}{}{}{}",
                                GrepInFile::COLOUR_FILE, self.file_path.display(), GrepInFile::COLOUR_END,
                                GrepInFile::COLOUR_LINE, self.line_number, GrepInFile::COLOUR_END,
                                &line[..m.start()], GrepInFile::COLOUR_MATCH, &line[m.range()], GrepInFile::COLOUR_END, &line[m.end()..]);
                            required_after = self.num_after;
                        }
                        None => {
                            if self.num_before > 0 {
                                self.before_lines.push_back(line.clone());
                                if self.before_lines.len() > self.num_before {
                                    self.before_lines.pop_front();
                                }
                            }
                        }
                    };

                    if required_after > 0 {
                        println!("{}{}{}:{}{}{}+ {}",
                            GrepInFile::COLOUR_FILE, self.file_path.display(), GrepInFile::COLOUR_END,
                            GrepInFile::COLOUR_LINE, self.line_number, GrepInFile::COLOUR_END, &line);
                        required_after -= 1;
                    }
                }
            }
        }

        Ok(())
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
