use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Ord, PartialOrd)]
#[serde(tag = "type")]
pub enum Output {
    Startup(Startup),
    Command(Command),
    Shutdown(Shutdown),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Ord, PartialOrd)]
pub struct Startup {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Ord, PartialOrd)]
pub struct Command {
    pub msg: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Ord, PartialOrd)]
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
            error: Some("you're smelly".to_string()),
        });

        println!("{}", serde_json::to_string_pretty(&output).unwrap());

        let output = Output::Command(Command {
            msg: "Hey there".to_string(),
        });

        println!("{}", serde_json::to_string_pretty(&output).unwrap());

        let output = Output::Shutdown(Shutdown {
            msg: "Hey there".to_string(),
        });

        println!("{}", serde_json::to_string_pretty(&output).unwrap());

        panic!("Expected");
    }
}
