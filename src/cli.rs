use clap::Command;
use std::error::Error;
use std::io::Write;
use std::io::{stdin, stdout};

use crate::action::Action;

pub async fn main(actions: &Vec<Action>) -> Result<(), Box<dyn Error>> {
    loop {
        let line = readline()?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match respond(actions, line).await {
            Ok(quit) => {
                if quit {
                    break;
                }
            }
            Err(err) => {
                write!(stdout(), "{err}")?;
                stdout().flush()?;
            }
        }
    }

    Ok(())
}

async fn respond(actions: &Vec<Action>, line: &str) -> Result<bool, Box<dyn Error>> {
    let args = shlex::split(line).ok_or("error: Invalid quoting")?;
    let matches = cli(actions)
        .try_get_matches_from(args)?;
    match matches.subcommand() {
        Some(("ping", _matches)) => {
            writeln!(stdout(), "Pong")?;
            stdout().flush()?;
        }
        Some(("quit", _matches)) => {
            writeln!(stdout(), "Exiting ...")?;
            stdout().flush()?;
            return Ok(true);
        }
        Some((name, _matches)) => {
            let action = actions.iter()
                .find(|action| action.cmd == name)
                .ok_or("error: Invalid command")?;
            action.invoke().await;
            stdout().flush()?;
        }
        None => unreachable!("subcommand required"),
    }


    Ok(false)
}

fn cli(actions: &Vec<Action>) -> Command {
    // strip out usage
    const PARSER_TEMPLATE: &str = "\
        {all-args}
    ";
    // strip out name/version
    const COMMAND_TEMPLATE: &str = "\
        {about-with-newline}\n\
        {usage-heading}\n    {usage}\n\
        \n\
        {all-args}{after-help}\
    ";

    let mut cmd = Command::new("repl")
        .multicall(true)
        .arg_required_else_help(true)
        .subcommand_required(true)
        .subcommand_value_name("COMMAND")
        .subcommand_help_heading("COMMANDS")
        .help_template(PARSER_TEMPLATE)
        .subcommand(
            Command::new("ping")
                .about("Get a response")
                .help_template(COMMAND_TEMPLATE),
        )
        .subcommand(
            Command::new("quit")
                .alias("exit")
                .alias("q")
                .alias(":q")
                .about("Quit the REPL")
                .help_template(COMMAND_TEMPLATE),
        );

    for action in actions.iter() {
        cmd = cmd.subcommand(
            Command::new(action.cmd)
                .about(action.desc)
                .help_template(COMMAND_TEMPLATE),
        );
    }

    cmd
}

fn readline() -> Result<String, Box<dyn Error>> {
    write!(stdout(), "> ")?;
    stdout().flush()?;
    let mut buffer = String::new();
    stdin()
        .read_line(&mut buffer)?;
    Ok(buffer)
}
