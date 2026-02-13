use std::{borrow::Cow, env, fs, path::Path, process::Command};
#[allow(unused_imports)]
use std::io::{self, Write};

use is_executable::IsExecutable;

const COMMAND: [&str; 5]= ["exit", "echo", "type", "pwd", "cd"];

#[derive(PartialEq, Default)]
enum CommandResult {
    Success,
    NotFound,
    #[default] CommandError
}

#[derive(Default)]
struct CommandExecutableResult {
    pub command: String,
    pub full_path: String,
    pub result: CommandResult,
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
        let mut command_args = "".to_string();
        if 1 < splited_command.len() {
            command_args = splited_command[1..].join(" ");
        }

        match command {
            "exit" => break,
            "echo" => command_echo(&command_args),
            "type" => command_type(&command_args),
            "pwd" => command_pwd(),
            "cd" => command_cd(&command_args),
            _ => command_execute(command, &command_args)
        };
    }
}

fn command_echo(args: &str) {
    println!("{}", args);
}

fn command_type(args: &str) {
    let check_command_executable_result = check_command_executable(args);
    if CommandResult::Success == check_command_executable_result.result {
        println!("{} is {}", check_command_executable_result.command, check_command_executable_result.full_path);
    }
}

fn command_pwd() {
    let Ok(current_path) = env::current_dir() else {
        println!("command_pwd current_dir error");
        return;
    };

    println!("{}", current_path.display());
}

fn command_cd(args: &str) {

    // cd HOME environment
    let try_change_path: Cow<'_, str> = if args == "~" {
        match env::var("HOME") {
            Ok(home) => Cow::Owned(home),
            Err(e) => {
                println!("command_cd HOME env cant found. error: {}", e);
                return;
            }
        }
    // else args path
    } else {
        Cow::Borrowed(args)
    };

    let change_path = Path::new(try_change_path.as_ref());
    let Ok(_) = env::set_current_dir(&change_path) else {
        println!("cd: {}: No such file or directory", args);
        return;
    };
}

fn command_execute(command: &str, command_args: &str) {
    let check_command_executable_result = check_command_executable(&format!("{} {}", command, command_args).to_string());
    if CommandResult::Success != check_command_executable_result.result {
        return;
    }

    // execute command
    match Command::new(check_command_executable_result.command).args(command_args.split_whitespace()).output() {
        Ok(output) => {
            print!("{}", str::from_utf8(&output.stdout).unwrap());
        },
        Err(e) => {
            println!("{}", e);
        }
    }
}

fn check_command_executable(args: &str) -> CommandExecutableResult {
    let mut result: CommandExecutableResult = CommandExecutableResult::default();

    if args.is_empty() {
        println!("command args error. args: {}", args);
        return result;
    }

    let splited_args: Vec<&str> = args.split(' ').collect();
    if splited_args.is_empty() || splited_args[0].is_empty() {
        println!("command args error. args: {}", args);
        return result;
    }

    let command = splited_args[0];

    result.command = command.to_string();
    result.result = CommandResult::NotFound;

    if COMMAND.contains(&command) {
        println!("{} is a shell builtin", command);
        return result;
    }

    // get PATH
    let Some(paths) = env::var_os("PATH") else {
        println!("{}: not found", command);
        return result;
    };

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

        result.full_path = full_display;
        result.result = CommandResult::Success;
        break;
    }

    if CommandResult::NotFound == result.result {
        println!("{}: not found", command);
    }

    return result;
}