use super::{
    input::InputCommand,
    output::{InputResponse, Output},
};
use std::io;
use yy_boss::YypBoss;

pub fn main_loop(mut yyp_boss: YypBoss) {
    let mut command = String::new();
    let stdin = io::stdin();

    loop {
        match stdin.read_line(&mut command) {
            Ok(_) => match serde_json::from_str::<InputCommand>(&command) {
                Ok(command) => {
                    println!("{:?}", command);
                }
                Err(e) => {
                    Output::Input(InputResponse {
                        msg: e.to_string(),
                        fatal: false,
                    })
                    .print();
                }
            },
            Err(e) => {
                Output::Input(InputResponse {
                    msg: e.to_string(),
                    fatal: false,
                })
                .print();
            }
        }

        command.clear();
    }
}
