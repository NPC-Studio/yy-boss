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
    output::{Output, Startup},
    yy_cli::YyCli,
};
use clap::{App, Arg};
use std::path::{Path, PathBuf};
use yy_boss::{StartupError, YypBoss};

/// The required
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Arguments {
    pub yyp_path: PathBuf,
    pub working_directory: PathBuf,
}

#[doc(hidden)]
pub(crate) fn parse_arguments() -> Result<Arguments, clap::Error> {
    let matches = App::new("Yy Boss")
        .version("0.3.1")
        .author("Jonathan Spira <jjspira@gmail.com>")
        .about("Manages a Gms2 project")
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
        .get_matches_safe();

    matches.map(|matches| {
        let yyp_path = Path::new(matches.value_of("path").unwrap()).to_owned();
        let working_directory = matches
            .value_of("working_directory")
            .map(|p| Path::new(p))
            .unwrap()
            .to_owned();

        Arguments {
            yyp_path,
            working_directory,
        }
    })
}

#[doc(hidden)]
pub(crate) fn startup(success: Result<YypBoss, StartupError>, yy_cli: &YyCli) -> Option<YypBoss> {
    let (yyp, error) = match success {
        Ok(yyp) => (Some(yyp), None),
        Err(err) => (None, Some(err)),
    };

    if error.is_some() {
        Output::Startup(Startup {
            success: yyp.is_some(),
            error: error.map(|e| e.to_string()),
        })
        .print();
        return None;
    } else if yy_cli.working_directory.is_dir() == false {
        Output::Startup(Startup {
            success: false,
            error: Some(StartupError::BadWorkingDirectoryPath.to_string()),
        })
        .print();
        return None;
    }

    Output::Startup(Startup {
        success: true,
        error: None,
    })
    .print();

    yyp
}
