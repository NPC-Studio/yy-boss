//! Right now, the only option which must be passed is the path to the Yyp.
//!
//! An example invocation of the cli would be:
//!
//! ```txt
//! ./yyp_boss_cli Documents/Projects/TestGms2Project/TestGms2Project.yyp
//! ```
//!
//! The above, assuming that `TestGms2Project` is parsed without error, will return
//! and output like below:
//! ```json
//! {
//!     "type": "Startup",
//!     "success": true
//! }
//! ```
//!
//! If the project does not parse correctly, it will return an error. For more on outputs,
//! see [`output`].
//!
//! In the future, we aim to support multiple forms of startup, including starting
//! with a non-serialized Yyp, and building up a full project over the cli. There are
//! numerous technical barriers in the architecture of `yy-boss` before that goal can
//! be achieved, but no serious barriers stand in the way.
//!
//! [`output`]: ../output/index.html

use super::{
    logging::Logging,
    output::{Output, Startup},
    yy_cli::YyCli,
};
use clap::{App, Arg};
use std::path::{Path, PathBuf};
use crate::{StartupError, YypBoss};

/// The required
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Arguments {
    pub yyp_path: PathBuf,
    pub working_directory: PathBuf,
    pub logging: Logging,
}

#[doc(hidden)]
pub(crate) fn parse_arguments() -> Result<Arguments, clap::Error> {
    let matches = App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about(clap::crate_description!())
        .version_short("v")
        .arg(
            Arg::with_name("path")
                .value_name("PATH")
                .required(true)
                .help("The path to the Yyp to load.")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("working_directory")
                .value_name("WORKING_DIRECTORY")
                .required(true)
                .help("the path to a safe working directory where the YypBoss will read and write")
                .long_help(
                    "A path to a safe working directory where the YypBoss will read and write. In the future \
                    we might support running the exe without a working dir.",
                )
                .takes_value(true),
        )
        .arg(
            Arg::with_name("log_file")
                .short("l")
                .takes_value(true)
                .required(false)
                .help("the path relative to the working directory where the YypBoss should place its logs.")
                .long_help(
                    "A path within the safe working directory where the YypBoss can place \
                    its logs. The YypBoss will ensure the path to the log location is valid (including making \
                    directories if necessary). By using \
                    \"log_stderr\", users can also log out to stderr instead."
                )
        )
        .arg(
            Arg::with_name("log_stderr")
                .short("s")
                .help("instructs the yypboss to output logs to stderr")
        )
        .get_matches_safe();

    matches.map(|matches| {
        let yyp_path = Path::new(matches.value_of("path").unwrap()).to_owned();
        let working_directory = matches
            .value_of("working_directory")
            .map(|p| Path::new(p))
            .unwrap()
            .to_owned();

        let logging = matches
            .value_of("log_file")
            .map(|p| Logging::LogToFile(Path::new(p).to_owned()))
            .unwrap_or_else(|| {
                if matches.values_of("log_stderr").is_some() {
                    Logging::LogToStdErr
                } else {
                    Logging::NoLog
                }
            });

        Arguments {
            yyp_path,
            working_directory,
            logging,
        }
    })
}

#[doc(hidden)]
pub(crate) fn startup(success: Result<YypBoss, StartupError>, yy_cli: &YyCli) -> Option<YypBoss> {
    let (yyp, error) = match success {
        Ok(yyp) => (Some(yyp), None),
        Err(err) => (None, Some(err)),
    };

    if let Some(error) = error {
        Output::Startup(Startup {
            success: false,
            error: Some(error.to_string()),
            project_metadata: None,
        })
        .print();
        return None;
    } else {
        std::fs::create_dir_all(&yy_cli.working_directory).ok()?;

        if yy_cli.working_directory.is_dir() == false {
            Output::Startup(Startup {
                success: false,
                error: Some(StartupError::BadWorkingDirectoryPath.to_string()),
                project_metadata: None,
            })
            .print();
            return None;
        }
    }

    Output::Startup(Startup {
        success: true,
        error: None,
        project_metadata: Some(yyp.as_ref().unwrap().project_metadata()),
    })
    .print();

    yyp
}
