use std::{env, fs};
#[allow(unused_imports)]
use std::io::{self, Write};

const COMMAND: [&str; 3]= ["exit", "echo", "type"];

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();

        io::stdin().read_line(&mut command).unwrap();
        
        command = command.trim().to_string();
        let splited_command: Vec<&str> = command.split(' ').collect();
        let command = splited_command[0];

        match command {
            "exit" => break,
            "echo" => command_echo( &splited_command[1..].join(" ")),
            "type" => command_type(splited_command[1]),
            _ => println!("{}: command not found", command)
        };
    }
}

fn command_echo(args: &str) {
    println!("{}", args);
}

fn command_type(args: &str) {
    if COMMAND.contains(&args) {
        println!("{} is a shell builtin", &args);
        return;
    }

    // get PATH
    let Some(paths) = env::var_os("PATH") else {
        println!("{}: not found", args);
        return;
    };

    let mut command_success = true;
    for path in env::split_paths(&paths) {
        let full = path.join(args);

        // Check if a file with the command name exists.
        let Ok(metadata) = fs::metadata(&full) else {
            continue
        };

        // Check if the file has execute permissions.
        if metadata.mode() & 0o111 != 0 {
            continue;
        }

        // If the file exists and has execute permissions, print <command> is <full_path> and stop.
        println!("{} is {}", &args, full.display());
        command_success = true;
        break;
    }

    // If no executable is found in any directory, print <command>: not found.
    if false == command_success {
        println!("{}: not found", &args);
    }
}