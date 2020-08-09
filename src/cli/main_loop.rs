use super::{
    input::{Command, ResourceCommandType},
    output::{InputResponse, Output},
};
use std::io;
use yy_boss::YypBoss;

pub fn main_loop(mut yyp_boss: YypBoss) {
    let mut command = String::new();
    let stdin = io::stdin();

    loop {
        match stdin.read_line(&mut command) {
            Ok(_) => {
                let output = match serde_json::from_str::<Command>(&command) {
                    Ok(command) => parse_command(command, &mut yyp_boss),
                    Err(e) => Output::Input(InputResponse {
                        msg: e.to_string(),
                        fatal: false,
                    }),
                };

                output.print();
            }
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

pub fn parse_command(command: Command, yyp_boss: &mut YypBoss) -> Output {
    match command {
        Command::Resource(resource_command) => match resource_command.command_type {
            ResourceCommandType::Add(new_resource) => unimplemented!(),
            ResourceCommandType::Replace(new_resource) => unimplemented!(),
            ResourceCommandType::Set(new_resource) => unimplemented!(),
            ResourceCommandType::Remove { identifier } => unimplemented!(),
            ResourceCommandType::Get { identifier } => unimplemented!(),
            ResourceCommandType::Exists { identifier } => unimplemented!(),
        },
        Command::VirtualFileSystem(vfs_command) => unimplemented!(),
    }
}
