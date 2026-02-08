#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();
        command = command.trim().to_string();
        let splited_command: Vec<&str> = command.split(' ').collect();
        let command = splited_command[0].to_string();

        if command == "exit" {
            break;
        }

        if command == "echo" {
            command_echo( &splited_command[1..].join(" "));
            continue;
        }

        println!("{}: command not found", command);
    }
}

fn command_echo(args: &str) {
    println!("{}", args);
}