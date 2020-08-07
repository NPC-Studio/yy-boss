use crate::errors::*;
use serde::{Deserialize, Serialize};
use yy_boss::YypBoss;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Output {
    Startup(Startup),
    Command(Command),
    Shutdown(Shutdown),
}

impl Output {
    pub fn startup(succes: Result<YypBoss, StartupError>) -> Option<YypBoss> {
        match succes {
            Ok(yyp) => {
                Output::Startup(Startup {
                    success: true,
                    error: None,
                })
                .print();

                Some(yyp)
            }
            Err(e) => {
                Output::Startup(Startup {
                    success: false,
                    error: Some(e),
                })
                .print();

                None
            }
        }
    }

    pub fn print(self) {
        let output = serde_json::to_string_pretty(&self).unwrap();
        println!("{}", output);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Startup {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<StartupError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Command {
    pub msg: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Shutdown {
    pub msg: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_serde() {
        let output = Output::Startup(Startup {
            success: false,
            error: Some(StartupError::BadPath),
        });

        output.print();

        panic!("Expected");
    }
}
