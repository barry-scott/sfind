use std::collections::VecDeque;
use std::path::PathBuf;

use std::fs;
use std::io::{BufRead, BufReader};

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

fn pattern_to_refex(pattern: &str, case_insensitive: bool) -> Result<Regex> {
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
            .map(|pattern| pattern_to_refex(pattern, opt.grep_ignore_case));

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
        let file = fs::File::open(self.file_path)?;
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
                    println!("find_match: {:?}", vec_m);
                }

                let mut line_number = self.line_number - self.before_lines.len();

                while let Some(line) = self.before_lines.pop_front() {
                    self.print_match_line(line_number, "-", &line);
                    line_number += 1;
                }
                let mut coloured_line = String::new();
                let mut last_end = 0;

                for m in &vec_m {
                    let mut m_start = m.start;
                    // deal with overlap
                    if m_start < last_end {
                        m_start = last_end;
                    }
                    let colour_index = m.pattern_index % GrepInFile::COLOUR_MATCH.len();

                    coloured_line.push_str(&line[last_end..m_start]);
                    coloured_line.push_str(GrepInFile::COLOUR_MATCH[colour_index]);
                    last_end = m.end;
                    coloured_line.push_str(&line[m_start..last_end]);
                    coloured_line.push_str(GrepInFile::COLOUR_END);
                }
                coloured_line.push_str(&line[last_end..]);

                self.print_match_line(self.line_number, ":", &coloured_line);

                required_after = self.num_after;
            }
        }

        Ok(())
    }

    const PADDING_SIZE: usize = 4;

    fn print_match_line(&self, line_number: usize, sep: &str, line: &str) {
        let path = self.file_path.display().to_string();
        let line_number = line_number.to_string();

        // len of path + ":" + 4 digits + sep + min-2-spaces
        let prefix_len_max = path.len() + 1 + 4 + 1 + 2;
        let prefix_len_min = path.len() + 1 + line_number.len() + 1 + 2;
        let padding_required = ((prefix_len_max + (GrepInFile::PADDING_SIZE - 1))
            % GrepInFile::PADDING_SIZE)
            * GrepInFile::PADDING_SIZE;

        let mut padding = String::new();
        for _ in 0..(prefix_len_max + padding_required - prefix_len_min) {
            if self.opt.debug {
                padding.push('·')
            } else {
                padding.push(' ')
            }
        }

        let mut match_report = String::new();
        match_report.push_str(GrepInFile::COLOUR_FILE);
        match_report.push_str(&path);
        match_report.push_str(GrepInFile::COLOUR_END);
        match_report.push(':');
        match_report.push_str(GrepInFile::COLOUR_LINE);
        match_report.push_str(&line_number);
        match_report.push_str(GrepInFile::COLOUR_END);
        match_report.push_str(sep);
        match_report.push_str(&padding);
        match_report.push_str(line);

        println!("{}", match_report);
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
