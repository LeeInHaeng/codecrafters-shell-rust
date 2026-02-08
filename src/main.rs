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

        if false == COMMAND.contains(&splited_command[1]) {
            println!("{}: not found", splited_command[1]);
            continue;
        }

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
    println!("{} is a shell builtin", args) ;
}