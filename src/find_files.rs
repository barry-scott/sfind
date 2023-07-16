use std::fs;
use std::collections::VecDeque;
use std::path::PathBuf;

pub use crate::command_options::CommandOptions as CommandOptions;
pub use crate::config_json::ConfigJson as ConfigJson;

struct PathToScan {
    pub path:       PathBuf,
    pub depth:      u32,
}

pub struct FindFiles<'caller> {
    folders:        VecDeque<PathToScan>,
    cur_dir_entry:  Option<fs::ReadDir>,
    cur_depth:      u32,
    opt:            &'caller CommandOptions,
    cfg:            &'caller ConfigJson,
}

impl PathToScan {
    pub fn new(path: PathBuf, depth: u32) -> PathToScan {
        PathToScan {
            path:   path,
            depth:  depth,
        }
    }
}

impl <'caller> Iterator for FindFiles<'caller> {
    type Item = PathBuf;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match &mut self.cur_dir_entry {
                // There is no cur_dir_entry in use
                // set to read_dir() of the next folder to scan
                None => {
                    match self.folders.pop_front() {
                        // no more folders to scan - then end!
                        None => {
                            return None;
                        }
                        // scan this folder
                        Some(path_to_scan) => {
                            match fs::read_dir(path_to_scan.path.clone()) {
                                Err(e) => {
                                    println!("error read_dir {} - {}", path_to_scan.path.display(), e);
                                    continue;
                                }
                                Ok(entry) => {
                                    self.cur_dir_entry = Some(entry);
                                    self.cur_depth = path_to_scan.depth;
                                    continue;
                                }
                            }
                        }
                    }
                }
                // use the cur_dir_entry that is active
                Some(dir_entry) => {
                    match dir_entry.next() {
                        // no more files in this dir
                        None => {
                            // set to None and try again on the next folder
                            self.cur_dir_entry = None;
                            continue;
                        }
                        Some(entry) => match entry {
                            // one more file or folder
                            Ok(entry) => {
                                let m = match entry.metadata() {
                                    Err(e) => {
                                        println!("error read_dir metadata {}", e);
                                        continue;
                                    }
                                    Ok(m) => m
                                };

                                // add this dir to the list of folders to be scanned
                                if m.is_dir() {
                                    // only go deeper if allowed.
                                    if match self.opt.find_depth {
                                        Some(depth) => {
                                            self.cur_depth < depth
                                        }
                                        // no limit on depth
                                        None => {
                                            true
                                        }
                                    } {
                                        self.folders.push_back(PathToScan::new(entry.path(), self.cur_depth+1));
                                    }
                                    continue;
                                }

                                // return the file that was found
                                return Some(entry.path());
                            }
                            // problem
                            Err(e) => {
                                println!("error read_dir next 2 {}", e);
                                continue;
                            }
                        }
                    }
                }
            }
        }
    }
}

impl <'caller> FindFiles<'caller> {
    pub fn new(opt: &'caller CommandOptions, cfg: &'caller ConfigJson) -> FindFiles<'caller> {
        let mut finder = FindFiles {
            folders:        VecDeque::new(),
            cur_dir_entry:  None,
            cur_depth:      0,
            opt:            opt,
            cfg:            cfg,
        };
        for path in &opt.folders {
            finder.folders.push_back(PathToScan::new(path.to_path_buf(), 1));
        }

        finder
    }
}
