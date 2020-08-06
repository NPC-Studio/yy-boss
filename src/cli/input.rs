use clap::{App, Arg};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    pub yyp_path: std::path::PathBuf,
}

pub fn parse_inputs() -> Input {
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

    Input {
        yyp_path: std::path::Path::new(path).to_owned(),
    }
}
