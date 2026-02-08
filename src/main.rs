use std::{env, fs, path::Path, process::Command};
#[allow(unused_imports)]
use std::io::{self, Write};

use is_executable::IsExecutable;

const COMMAND: [&str; 3]= ["exit", "echo", "type"];

#[derive(PartialEq)]
enum CommandResult {
    Success,
    NotFound,
    CommandError
}

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();

        io::stdin().read_line(&mut command).unwrap();
        
        command = command.trim().to_string();
        let splited_command: Vec<&str> = command.split(' ').collect();
        if splited_command.is_empty() || splited_command[0].is_empty() {
            continue;
        }

        let command = splited_command[0];
        let command_args = &splited_command[1..].join(" ");

        match command {
            "exit" => break,
            "echo" => command_echo(&command_args),
            "type" => command_type(&command_args),
            _ => println!("{}: command not found", command)
        };
    }
}

fn command_echo(args: &str) {
    println!("{}", args);
}

fn command_type(args: &str) {
    let splited_args: Vec<&str> = args.split(' ').collect();
    let command = splited_args[0];

    if COMMAND.contains(&command) {
        println!("{} is a shell builtin", command);
        return;
    }

    // get PATH
    let Some(paths) = env::var_os("PATH") else {
        println!("{}: not found", command);
        return;
    };

    let mut command_result = CommandResult::NotFound;
    for path in env::split_paths(&paths) {
        let full = path.join(command);

        // Check if a file with the command name exists.
        if fs::metadata(&full).is_err() {
            continue;
        }

        // Check if the file has execute permissions.
        let full_display = full.to_string_lossy().into_owned();
        let path_display = Path::new(&full_display);
        if false == path_display.is_executable() {
            continue;
        }

        println!("{} is {}", command, full_display);
        command_result = CommandResult::Success;

        // execute command
        /*
        match Command::new(full_display).args(&splited_args[1..]).output() {
            Ok(output) => {
                println!("{}", str::from_utf8(&output.stdout).unwrap());
                command_result = 
                break;
            },
            Err(e) => {
                println!("{}", e);
                command_result = CommandResult::CommandError;
                break;
            }
        }
         */
    }

    // If no executable is found in any directory, print <command>: not found.
    if CommandResult::NotFound == command_result {
        println!("{}: not found", &args);
    }
}