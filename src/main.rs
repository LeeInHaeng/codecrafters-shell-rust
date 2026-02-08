use std::{env, path, process::Command};
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
    let paths: Vec<String> = match env::var_os("PATH") {
        Some(p) => env::split_paths(&p)
            .map(|path| path.to_string_lossy().into_owned())
            .collect(),
        None => {
            println!("{}: not found", args);
            Vec::new()
        }
    };

    let mut command_success = false;
    for path in paths {
        let last_path = match path.split('/').last() {
            Some(v) => v,
            None => continue
        };

        if args != last_path {
            continue;
        }

        // Check if the file has execute permissions.
        // If the file exists and has execute permissions, print <command> is <full_path> and stop.
        // If the file exists but lacks execute permissions, skip it and continue to the next directory.
        let command_result = Command::new(&path).output();
        match command_result {
            Ok(_) => {
                println!("{} is {}", &args, &path);
                command_success = true;
                break;
            },
            Err(_) => continue
        };
    }

    if false == command_success {
        println!("{}: not found", &args);
    }
}