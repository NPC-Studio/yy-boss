use crate::errors::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Output {
    Startup(Startup),
    Input(InputResponse),
    Command(Command),
    Shutdown(Shutdown),
}

impl Output {
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

#[derive(Debug, Serialize, Deserialize)]
pub struct InputResponse {
    pub msg: String,
    pub fatal: bool,
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
