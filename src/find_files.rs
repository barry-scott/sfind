use std::collections::VecDeque;
use std::fs::{self, DirEntry, Metadata};
use std::path::{Path, PathBuf};

use cfg_if;

use regex::{Regex, RegexBuilder};

pub use crate::command_options::CommandOptions;
pub use crate::config_json::ConfigJson;

cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
    } else {
        // macOS and linux
        use std::os::linux::fs::MetadataExt;

        // avoid using the libc crate just for this def
        const S_IFREG: u32 = 32768;
    }
}

#[derive(Debug)]
struct PathToScan {
    pub path: PathBuf,
    pub depth: usize,
}

pub struct FindFiles<'caller> {
    folders: VecDeque<PathToScan>,
    cur_dir_entry: Option<fs::ReadDir>,
    cur_depth: usize,
    opt: &'caller CommandOptions,
    folders_to_prune: Option<Regex>,
    files_to_prune: Option<Regex>,
    files_to_find: Option<Regex>,
}

impl PathToScan {
    pub fn new(path: PathBuf, depth: usize) -> PathToScan {
        PathToScan { path, depth }
    }
}

macro_rules! continue_on_err {
    ($report:expr, $val:expr, $msg:literal, $file:expr) => {
        match $val {
            Ok(v) => v,
            Err(e) => {
                if($report) {
                    eprintln!($msg, $file, e);
                }
                continue;
            }
        }
    };
}

impl<'caller> Iterator for FindFiles<'caller> {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.cur_dir_entry()?.next() {
                // no more files in this dir
                None => {
                    // set to None and try again on the next folder
                    self.cur_dir_entry = None;
                    continue;
                }
                Some(entry) => {
                    let entry = continue_on_err!(true, entry, "error read_dir next 2 {} - {}", "");
                    let m = continue_on_err!(true, entry.metadata(), "error read_dir metadata {} - {}", entry.path().display());

                    let is_dir = if m.is_symlink() {
                        let m = continue_on_err!(self.opt.report_supressed_errors, fs::metadata(entry.path()), "error read_dir os symlink {} - {}", entry.path().display());
                        m.is_dir()
                    } else {
                        m.is_dir()
                    };

                    // add this dir to the list of folders to be scanned
                    if is_dir {
                        // only go deeper if allowed.
                        if self.go_deeper() {
                            self.push_folder(PathToScan::new(entry.path(), self.cur_depth + 1));
                        }
                        continue;
                    };

                    if self.return_file(&entry, &m) {
                        break Some(entry.path());
                    }
                }
            }
        }
    }
}

// Utility functions for the iterator implementation
impl<'caller> FindFiles<'caller> {
    fn cur_dir_entry(&mut self) -> Option<&mut std::fs::ReadDir> {
        // There is no cur_dir_entry in use
        // set to read_dir() of the next folder to scan
        while self.cur_dir_entry.is_none() {
            // In a function returning `Option`, `?` returns if we get a `None`.
            let path_to_scan = self.folders.pop_front()?;

            match fs::read_dir(path_to_scan.path.clone()) {
                Err(e) => {
                    eprintln!("Error: read_dir {} - {}", path_to_scan.path.display(), e);
                    continue;
                }
                Ok(entry) => {
                    self.cur_dir_entry = Some(entry);
                    self.cur_depth = path_to_scan.depth;
                    break;
                }
            }
        }

        self.cur_dir_entry.as_mut()
    }

    fn push_folder(&mut self, path_to_scan: PathToScan) {
        if self.exclude_folder(&path_to_scan.path) {
            if self.opt.debug {
                eprintln!("Debug: exclude folder {:?}", path_to_scan);
            }
        } else {
            self.folders.push_back(path_to_scan)
        }
    }

    fn go_deeper(&self) -> bool {
        self.opt
            .find_depth
            .map(|depth| self.cur_depth < depth)
            .unwrap_or(true)
    }

    fn return_file(&self, entry: &DirEntry, m: &Metadata) -> bool {
        // avoid scanning fifo, etc
        if !Self::file_is_regular(m) {
            return false
        }

        if self.files_to_find.is_some() {
            if !self.include_file(entry) {
                false
            } else {
                if self.opt.debug {
                    eprintln!("Debug: include_file {:?}", entry.path());
                }
                self.file_time_allowed(m) && self.file_size_allowed(m)
            }
        } else {
            // exclude files that are config to be pruned
            if self.exclude_file(entry) {
                if self.opt.debug {
                    eprintln!("Debug: exclude_file {:?}", entry.path());
                }
                false
            } else {
                if self.opt.debug {
                    eprintln!("Debug: file not included or excluded {:?}", entry.path());
                }
                self.file_time_allowed(m) && self.file_size_allowed(m)
            }
        }
    }

    cfg_if::cfg_if! {
        if #[cfg(target_os = "windows")] {
            fn file_is_regular(m: &Metadata) {
                return !m.is_dir()
            }
        } else {
            // macOS and linux

            fn file_is_regular(m: &Metadata) -> bool {
                (m.st_mode() & S_IFREG) != 0
            }
        }
    }

    fn file_size_allowed(&self, m: &Metadata) -> bool {
        match self.opt.size_min {
            None => {}
            Some(size_min) => {
                    // is the file too small?
                if m.len() < size_min {
                    return false
                }
            }
        }
        match self.opt.size_max {
            None => {}
            Some(size_max) => {
                    // is the file too big?
                if m.len() > size_max {
                    return false
                }
            }
        }
        true
    }

    fn file_time_allowed(&self, m: &Metadata) -> bool {
        match self.opt.time_from {
            None => {
                // no check required
                return true
            }
            Some(time_from) => {
                // check file times
                match m.modified().ok() {
                    None => {
                        // skip time check
                        return true;
                    }

                    Some(file_mod_secs) => {
                        // is the file too old?
                        if time_from < file_mod_secs {
                            match self.opt.time_till {
                                Some(time_until) => {
                                    // is the file too new?
                                    return file_mod_secs < time_until;
                                }
                                None => {
                                    return true
                                }
                            }
                        } else {
                            return false
                        }
                    }
                }
            }
        }
    }
}

impl<'caller> FindFiles<'caller> {
    pub fn new(opt: &'caller CommandOptions, cfg: &'caller ConfigJson) -> FindFiles<'caller> {
        let mut finder = FindFiles {
            folders: VecDeque::new(),
            cur_dir_entry: None,
            cur_depth: 0,
            opt,
            folders_to_prune: FindFiles::match_filenames_regex(&cfg.folders_to_prune, true),
            files_to_prune: FindFiles::match_filenames_regex(&cfg.files_to_prune, true),
            files_to_find: FindFiles::match_filenames_regex(&opt.files, opt.find_iname),
        };
        for path in &opt.folders {
            finder
                .folders
                .push_back(PathToScan::new(path.to_path_buf(), 1));
        }

        finder
    }

    fn exclude_folder(&self, folder_path: &Path) -> bool {
        match &self.folders_to_prune {
            Some(regex) => {
                let folder_name = match folder_path.file_name() {
                    Some(file_name) => file_name,
                    None => {
                        if folder_path.to_str() != Some(".") && folder_path.to_str() != Some("..") {
                            return false;
                        }
                        folder_path.as_os_str()
                    }
                };
                match folder_name.to_str() {
                    Some(folder_name) => {
                        let exclude = regex.is_match(folder_name);
                        if self.opt.debug {
                            eprintln!("Debug: exclude {} -> {:?}", folder_name, exclude);
                        }
                        exclude
                    }
                    None => {
                        eprintln!("Error: folder_name is not utf-8");
                        true
                    }
                }
            }
            None => false,
        }
    }

    fn match_file(&self, match_regex: Option<&Regex>, entry: &fs::DirEntry, match_basename: bool) -> bool {
        match_regex
            .map(|regex| {
                if let Ok(path_or_file_name) =
                        if match_basename {
                            entry.file_name().into_string()
                        } else {
                            entry.path().into_os_string().into_string()
                        }
                    {
                    //eprintln!("QQQ path_or_file_name {}", path_or_file_name);
                    //eprintln!("QQQ regex {}", regex);
                    regex.is_match(&path_or_file_name)
                } else {
                    eprintln!("Error: file_name is not utf-8 {}", entry.path().display());
                    false
                }
            })
            .unwrap_or(false)
    }

    fn include_file(&self, entry: &fs::DirEntry) -> bool {
        self.match_file(self.files_to_find.as_ref(), entry, self.opt.find_match_basename)
    }

    fn exclude_file(&self, entry: &fs::DirEntry) -> bool {
        self.match_file(self.files_to_prune.as_ref(), entry, true)
    }

    fn match_filenames_regex(all_patterns: &[String], case_insensitive: bool) -> Option<Regex> {
        if all_patterns.is_empty() {
            None
        } else {
            let mut prune_pattern = String::new();
            prune_pattern.push_str("^(");
            let mut sep = "";

            for pattern in all_patterns {
                prune_pattern.push_str(sep);
                prune_pattern.push_str(&FindFiles::glob_pattern_to_regex_pattern(pattern));
                sep = "|";
            }
            prune_pattern.push_str(")$");
            match RegexBuilder::new(&prune_pattern)
                .case_insensitive(case_insensitive)
                .build()
            {
                Ok(regex) => Some(regex),
                Err(e) => {
                    eprintln!("Error: bad pattern {} - {}", prune_pattern, e);
                    None
                }
            }
        }
    }

    fn glob_pattern_to_regex_pattern(glob_pattern: &str) -> String {
        let mut regex_pattern = String::new();

        for ch in glob_pattern.chars() {
            match ch {
                '*' => regex_pattern.push_str(".*"),
                '?' => regex_pattern.push('.'),
                '.' | '+' | '(' | ')' | '|' | '\\' | '[' | ']' | '{' | '}' | '^' | '$' | '#' => {
                    regex_pattern.push('\\');
                    regex_pattern.push(ch)
                }
                _ => regex_pattern.push(ch),
            };
        }

        regex_pattern
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn glob_pattern_to_regex_pattern() {
        assert_eq!(FindFiles::glob_pattern_to_regex_pattern("fixed"), "fixed");
        assert_eq!(
            FindFiles::glob_pattern_to_regex_pattern("file.type"),
            "file\\.type"
        );
        assert_eq!(
            FindFiles::glob_pattern_to_regex_pattern("*.type"),
            ".*\\.type"
        );
    }

    #[test]
    fn regex_match() {
        let regex = Regex::new("^a.*$").unwrap();
        let haystack = String::from("abc.txt");
        assert!(regex.is_match(&haystack));

        let regex = Regex::new("^(a.*)$").unwrap();
        let haystack = String::from("abc.txt");
        assert!(regex.is_match(&haystack));

        let regex = Regex::new("^(a.*|b.*)$").unwrap();
        let haystack = String::from("abc.txt");
        assert!(regex.is_match(&haystack));

        let regex = Regex::new("^(.*fedora.*)$").unwrap();
        let haystack = String::from(".password-store/fedoraproject.org/otp.gpg");
        assert!(regex.is_match(&haystack));
    }

    #[test]
    fn regex_vec_match() {
        let glob_patterns = vec![String::from("*.txt")];
        let regex = FindFiles::match_filenames_regex(&glob_patterns, false).unwrap();
        assert_eq!(regex.as_str(), r#"^(.*\.txt)$"#);

        let haystack = String::from("abc.txt");
        assert!(regex.is_match(&haystack));

        let glob_patterns = vec![String::from("*.txt"), String::from("*.rs")];
        let regex = FindFiles::match_filenames_regex(&glob_patterns, false).unwrap();
        assert_eq!(regex.as_str(), r#"^(.*\.txt|.*\.rs)$"#);

        assert!(regex.is_match(&haystack));

        let haystack = String::from("abc.rs");
        assert!(regex.is_match(&haystack));

        let haystack = String::from("abc.toml");
        assert!(!regex.is_match(&haystack));
    }
}
