use std::collections::VecDeque;
use std::path::PathBuf;

use std::fs;
use std::io::{BufRead, BufReader};

use anyhow::{Result, anyhow};
use regex::{Regex, RegexBuilder};

pub use crate::command_options::CommandOptions as CommandOptions;

pub struct GrepPatterns {
    pub patterns:        Vec<Regex>,
}

#[derive(Debug)]
pub struct GrepMatch {
    pattern_index:  usize,
    start:          usize,
    end:            usize,
}

pub struct GrepInFile<'caller> {
    opt:                &'caller CommandOptions,
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

    pub fn find_match(&'caller self, line: &'caller str) -> Option<Vec<GrepMatch>> {
        let mut matches = Vec::new();

        for (index, regex) in self.patterns.iter().enumerate() {
            // assuming here that is_match is faster then find
            if matches.len() == 0 {
                if regex.is_match(&line) {
                    // add the first match to the vec.
                    for m in regex.find_iter(&line) {
                        matches.push(GrepMatch { pattern_index: index, start: m.start(), end: m.end() })
                    }
                }
            } else {
                // look for more matches to add
                for m in regex.find_iter(&line) {
                    matches.push(GrepMatch { pattern_index: index, start: m.start(), end: m.end() })
                }
            }
        }
        if matches.len() > 0 {
            matches.sort_by(|a, b| a.start.cmp(&b.start));
            Some(matches)
        } else {
            None
        }
    }

    fn quote_regex(text: &str) -> String {
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
            opt:                opt,
            patterns:           patterns,
            file_path:          file_path,
            num_before:         match opt.grep_lines_before { Some(n) => n, None => 0 },
            before_lines:       VecDeque::new(),
            line_number:        0,
            num_after:          match opt.grep_lines_after { Some(n) => n, None => 0 },
        };

        gif
    }

    const COLOUR_FILE: &str = "\x1b[35m";   // purple
    const COLOUR_LINE: &str = "\x1b[32m";   // green
    const COLOUR_MATCH: &'static [&'static str] = &[
        "\x1b[1;31m",   // light red
        "\x1b[33m",     // yellow
        "\x1b[1;34m",   // light blue
        "\x1b[32m",     // green
        "\x1b[35m",     // purple
        ];
    const COLOUR_END: &str = "\x1b[m";      // no colour

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
                        Some(vec_m) => {
                            if self.opt.debug {
                                println!("find_match: {:?}", vec_m);
                            }

                            let mut line_number = self.line_number - self.before_lines.len();

                            loop {
                                match self.before_lines.pop_front() {
                                    Some(line) => {
                                        self.print_match_line(line_number, "-", &line);
                                        line_number += 1;
                                    },
                                    None => break
                                }
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
                            coloured_line.push_str(&line[last_end..line.len()]);

                            self.print_match_line(self.line_number, ":", &coloured_line);

                            required_after = self.num_after;
                        }
                        None => {
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
                        }
                    }
                }
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
        let padding_required = ((prefix_len_max + (GrepInFile::PADDING_SIZE-1)) % GrepInFile::PADDING_SIZE) * GrepInFile::PADDING_SIZE;

        let mut padding = String::new();
        for _ in 0..(prefix_len_max + padding_required - prefix_len_min) {
            if self.opt.debug {
                padding.push_str("·")
            } else {
                padding.push_str(" ")
            }
        };

        let mut match_report = String::new();
        match_report.push_str(GrepInFile::COLOUR_FILE);
        match_report.push_str(&path);
        match_report.push_str(GrepInFile::COLOUR_END);
        match_report.push_str(":");
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
