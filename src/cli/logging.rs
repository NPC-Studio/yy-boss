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

pub fn begin_logging(log: Logging, wd: &Path) -> Result<(), anyhow::Error> {
    match log {
        Logging::NoLog => Ok(()),
        Logging::LogToFile(pathbuf) => {
            let output = if pathbuf.is_relative() {
                wd.join(pathbuf)
            } else {
                pathbuf
            };

            std::fs::create_dir_all(
                output
                    .parent()
                    .ok_or_else(|| anyhow::anyhow!("don't use root directories you idiot"))?,
            )?;

            log_to_file(output, LevelFilter::Info)?;

            Ok(())
        }
        Logging::LogToStdErr => {
            log_to_stderr(LevelFilter::Info);
            Ok(())
        }
    }
}
