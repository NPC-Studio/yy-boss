use log::LevelFilter;
use simple_logging::{log_to_file, log_to_stderr};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub enum Logging {
    NoLog,
    LogToFile(PathBuf),
    LogToStdErr,
}

impl Default for Logging {
    fn default() -> Self {
        Self::NoLog
    }
}

pub fn begin_logging(log: Logging, wd: &Path) {
    match log {
        Logging::NoLog => {}
        Logging::LogToFile(pathbuf) => {
            let output = wd.join(pathbuf);
            std::fs::create_dir_all(output.parent().unwrap()).expect("oh no logging broke");

            log_to_file(output, LevelFilter::Info).expect("oh no logging broke");
        }
        Logging::LogToStdErr => {
            log_to_stderr(LevelFilter::Info);
        }
    }
}
