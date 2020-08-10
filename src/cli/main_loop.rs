use super::{
    input::Command,
    output::{CommandOutput, Output, YypBossError::CouldNotReadCommand},
    yy_cli::YyCli,
};
use std::io;
use yy_boss::YypBoss;

pub fn main_loop(mut yyp_boss: YypBoss, yy_cli: YyCli) {
    let mut command = String::new();
    let stdin = io::stdin();

    loop {
        match stdin.read_line(&mut command) {
            Ok(_) => {
                let output = match serde_json::from_str::<Command>(&command) {
                    Ok(command) => yy_cli.parse_command(command, &mut yyp_boss),
                    Err(e) => {
                        Output::Command(CommandOutput::error(CouldNotReadCommand(e.to_string())))
                    }
                };

                output.print();
            }
            Err(e) => {
                let mut output = CommandOutput::error(CouldNotReadCommand(e.to_string()));
                output.fatal = Some(true);
                Output::Command(output).print();

                break;
            }
        }

        command.clear();
    }
}
