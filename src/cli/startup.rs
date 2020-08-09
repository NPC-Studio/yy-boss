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

use super::output::{Output, Startup};
use clap::{App, Arg};
use yy_boss::{errors::StartupError, YypBoss};

/// The required
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Arguments {
    pub yyp_path: std::path::PathBuf,
}

#[doc(hidden)]
pub(crate) fn parse_arguments() -> Arguments {
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
        .get_matches();

    let path = matches.value_of("path").unwrap();

    Arguments {
        yyp_path: std::path::Path::new(path).to_owned(),
    }
}

#[doc(hidden)]
pub(crate) fn startup(success: Result<YypBoss, StartupError>) -> Option<YypBoss> {
    let (yyp, error) = match success {
        Ok(yyp) => (Some(yyp), None),
        Err(err) => (None, Some(err)),
    };

    Output::Startup(Startup {
        success: yyp.is_some(),
        error,
    })
    .print();

    yyp
}
