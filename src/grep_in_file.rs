use std::collections::VecDeque;
use std::path::PathBuf;

use std::io::{BufRead, BufReader, Read, Seek};
use std::{fs, iter, mem};

use anyhow::{anyhow, Result};
use regex::{Regex, RegexBuilder};

pub use crate::command_options::CommandOptions;

pub struct GrepPatterns {
    pub patterns: Vec<Regex>,
}

#[derive(Debug)]
pub struct GrepMatch {
    pattern_index: usize,
    start: usize,
    end: usize,
}

pub struct GrepInFile<'caller> {
    opt: &'caller CommandOptions,
    patterns: &'caller GrepPatterns,
    file_path: &'caller PathBuf,
    num_before: usize,
    before_lines: VecDeque<String>,
    line_number: usize,
    num_after: usize,
}

fn fixed_to_regex(fixed: &str, case_insensitive: bool) -> Result<Regex> {
    RegexBuilder::new(&GrepPatterns::quote_regex(fixed))
        .case_insensitive(case_insensitive)
        .build()
        .map_err(|_| anyhow!("failed to compile regex for {}", fixed))
}

fn pattern_to_regex(pattern: &str, case_insensitive: bool) -> Result<Regex> {
    RegexBuilder::new(pattern)
        .case_insensitive(case_insensitive)
        .build()
        .map_err(|_| anyhow!("failed to compile regex for {}", pattern))
}

impl GrepPatterns {
    pub fn new(opt: &CommandOptions) -> Result<GrepPatterns> {
        let fixed = opt
            .fixed_strings
            .iter()
            .map(|fixed| fixed_to_regex(fixed, opt.grep_ignore_case));
        let regex = opt
            .regex_patterns
            .iter()
            .map(|pattern| pattern_to_regex(pattern, opt.grep_ignore_case));

        let patterns: Result<Vec<_>> = fixed.chain(regex).collect();
        let patterns = patterns?;

        Ok(GrepPatterns { patterns })
    }

    pub fn find_match(&self, line: &str) -> Vec<GrepMatch> {
        let mut matches: Vec<_> = self
            .patterns
            .iter()
            .enumerate()
            .flat_map(|(pattern_index, regex)| {
                regex.find_iter(line).map(move |m| GrepMatch {
                    pattern_index,
                    start: m.start(),
                    end: m.end(),
                })
            })
            .collect();

        matches.sort_by(|a: &GrepMatch, b: &GrepMatch| a.start.cmp(&b.start));
        matches
    }

    fn quote_regex(text: &str) -> String {
        let mut regex_pattern = String::new();

        for ch in text.chars() {
            match ch {
                '.' | '+' | '*' | '?' | '#' | '^' | '$' | '\\' | '(' | ')' | '|' | '[' | ']'
                | '{' | '}' => {
                    regex_pattern.push('\\');
                    regex_pattern.push(ch);
                }
                _ => regex_pattern.push(ch),
            }
        }

        regex_pattern
    }
}

fn colour_match_line(line: &str, matches: &[GrepMatch]) -> String {
    let colour_codes_len = GrepInFile::COLOUR_MATCH[0].len() + GrepInFile::COLOUR_END.len();
    let mut coloured_line = String::with_capacity(line.len() + colour_codes_len * matches.len());
    let mut last_end = 0;

    for m in matches {
        // deal with overlap
        let m_start = if m.start < last_end {
            last_end
        } else {
            m.start
        };
        let colour_index = m.pattern_index % GrepInFile::COLOUR_MATCH.len();

        coloured_line.push_str(&line[last_end..m_start]);
        coloured_line.push_str(GrepInFile::COLOUR_MATCH[colour_index]);
        coloured_line.push_str(&line[m_start..m.end]);
        coloured_line.push_str(GrepInFile::COLOUR_END);
        last_end = m.end;
    }
    coloured_line.push_str(&line[last_end..]);

    coloured_line
}

impl<'caller> GrepInFile<'caller> {
    pub fn new(
        opt: &'caller CommandOptions,
        file_path: &'caller PathBuf,
        patterns: &'caller GrepPatterns,
    ) -> GrepInFile<'caller> {
        GrepInFile {
            opt,
            patterns,
            file_path,
            num_before: opt.grep_lines_before.unwrap_or(0),
            before_lines: VecDeque::new(),
            line_number: 0,
            num_after: opt.grep_lines_after.unwrap_or(0),
        }
    }

    const COLOUR_FILE: &str = "\x1b[35m"; // purple
    const COLOUR_LINE: &str = "\x1b[32m"; // green
    const COLOUR_MATCH: &'static [&'static str] = &[
        "\x1b[1;31m", // light red
        "\x1b[33m",   // yellow
        "\x1b[1;34m", // light blue
        "\x1b[32m",   // green
        "\x1b[35m",   // purple
    ];
    const COLOUR_END: &str = "\x1b[m"; // no colour

    pub fn search(&mut self) -> Result<()> {
        let mut file = fs::File::open(self.file_path)?;
        let mut bin_check_buf = [0u8; 1024];
        let check_len = file.read(&mut bin_check_buf[..])?;

        // is first line possibly binary?
        for byte in &mut bin_check_buf[0..check_len] {
            match byte {
                0 | 1 => {
                    if self.opt.debug {
                        eprintln!("Debug: assuming binary file {}", self.file_path.display())
                    }
                    return Ok(());
                }
                _ => {
                    continue;
                }
            }
        }

        file.rewind()?;

        let reader = BufReader::new(file);

        let mut required_after = 0;

        for line_result in reader.lines() {
            let line = line_result
                .map_err(|e| anyhow!("Error reading {} - {}", self.file_path.display(), e))?;
            self.line_number += 1;

            let vec_m = self.patterns.find_match(&line);
            if vec_m.is_empty() {
                // No matches
                if self.num_before > 0 {
                    self.before_lines.push_back(line.clone());
                    if self.before_lines.len() > self.num_before {
                        self.before_lines.pop_front();
                    }
                }
                if required_after > 0 {
                    self.print_match_line(self.line_number, "+", &line);
                    required_after -= 1;
                }
            } else {
                if self.opt.debug {
                    eprintln!("Debug: find_match: {:?}", vec_m);
                }

                let before_lines = mem::take(&mut self.before_lines);
                let line_number_base = self.line_number - before_lines.len();

                for (offset, before_line) in before_lines.into_iter().enumerate() {
                    self.print_match_line(line_number_base + offset, "-", &before_line);
                }

                self.print_match_line(self.line_number, ":", &colour_match_line(&line, &vec_m));

                required_after = self.num_after;
            }
        }

        Ok(())
    }

    const PADDING_SIZE: usize = 4;

    fn print_match_line(&self, line_number: usize, sep: &str, line: &str) {
        let path = self.file_path.display().to_string();
        let line_number = line_number.to_string();

        // len of path + ":" + min 4 digits + sep + min-2-spaces
        let prefix_len = path.len() + 1 + std::cmp::max(4, line_number.len()) + 1 + 2;
        let padding_required = GrepInFile::PADDING_SIZE - (prefix_len % GrepInFile::PADDING_SIZE);

        let padding: String = iter::repeat(if self.opt.debug { '·' } else { ' ' })
            .take(padding_required)
            .collect();

        println!(
            "{colour_file}{path}{colour_end}:{colour_line}{line_number}{colour_end}{sep}{padding}{line}",
            colour_file = GrepInFile::COLOUR_FILE,
            colour_end = GrepInFile::COLOUR_END,
            colour_line = GrepInFile::COLOUR_LINE
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quote_regex() {
        assert_eq!(GrepPatterns::quote_regex("fixed"), "fixed");
        assert_eq!(GrepPatterns::quote_regex("file.type"), "file\\.type");
        assert_eq!(GrepPatterns::quote_regex("*.type"), "\\*\\.type");
    }
}
