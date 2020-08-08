use super::{Output, Startup};
use clap::{App, Arg};
use yy_boss::{errors::StartupError, YypBoss};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Arguments {
    pub yyp_path: std::path::PathBuf,
}

pub fn parse_arguments() -> Arguments {
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

pub fn startup(success: Result<YypBoss, StartupError>) -> Option<YypBoss> {
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
