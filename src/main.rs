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

        // owned
        let mut command = String::new();

        io::stdin().read_line(&mut command).unwrap();
        
        // 아래 라인 부터는 owned 가 불필요 , borrow 로 진행
        let mut command = command.trim();
        let command_args;

        if let Some((cmd, rest)) = command.split_once(' ') {
            command = cmd;
            command_args = rest.trim();
        } else {
            command_args = "";
        }

        match command {
            // 파라미터가 불필요한 명령어
            "exit" => break,
            "pwd" => command_pwd(),
            // 파라미터가 필요한 명령어
            _ => {
                match command {
                    "echo" => command_echo(&command_args),
                    "type" => command_type(&command_args),
                    "cd" => command_cd(&command_args),
                    // 해당 challenge 에서 cat 명령어가 기본적으로 있다고 하지만
                    // 윈도우 환경 에서 정상 인식이 안되기 때문에 별도 command 로 구현
                    "cat" => command_cat(&command_args),
                    _ => command_execute(command, &command_args)
                };
            }
        };
    }
}

//// single quotes ///
// 'hello    world'   :   hello    world  : 따옴표 안의 공백은 그대로 유지됩니다.
// hello    world :   hello world : 연속된 공백은 따옴표로 묶지 않는 한 축소됩니다.
// 'hello''world' : helloworld    : 인접한 따옴표로 묶인 문자열 'hello'은 'world'연결됩니다.
// hello''world   : helloworld    : 빈 따옴표 ''는 무시됩니다.

/// double quotes ///
// "hello    world" : hello    world
// "hello""world" : helloworld
// "hello" "world" : hello world
// "shell's test" : shell's test
fn command_args_builder(args: &str) -> String {
    let mut result = String::with_capacity(args.len());
    let mut is_quote_start = false;
    let mut is_double_quote = false;

    for (idx, char) in args.char_indices() {
        if char == '\'' || char == '\"' {
            if is_quote_start {
                // double quotes 로 묶인거면 single quotes 는 string 에 담고 무시
                if is_double_quote && char == '\'' {
                    result.push(char);
                    continue;
                }

                is_quote_start = false;
                is_double_quote = false;
                continue;
            } else {
                is_quote_start = true;
                if char == '\"' {
                    is_double_quote = true;
                }

                continue;
            }
        } else {
            // 쿼터가 시작 단계 였으면 아무 가공도 하지 않고 그냥 push
            if is_quote_start {
                result.push(char);
                continue;
            // 쿼터로 묶인 단계가 아닐 경우
            } else {
                let mut before_char = '\0';
                if 0 < idx {
                    before_char = args.as_bytes()[idx - 1] as char;
                }

                // 현재 char 이 공백이고, 이전 인덱스의 char 가 공백이면 중복 공백 제거를 위해 pass
                if char == ' ' && before_char == ' ' {
                    continue;
                // string push
                } else {
                    result.push(char);
                    continue;
                }
            }
        }
    }
    result
}

fn command_echo(args: &str) {
    println!("{}", command_args_builder(args));
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

fn command_cat(args: &str) {
    let file_path_args: Vec<&str> = args.split('\'')
        .filter(|s| !s.trim().is_empty() && s.trim() != " ")
        .collect();

    for file_path in file_path_args {
        let mut quoted_path = String::with_capacity(file_path.len() + 2);
        quoted_path.push('\'');
        quoted_path.push_str(file_path);
        quoted_path.push('\'');

        quoted_path = command_args_builder(&quoted_path);
        let Ok(file_contents) = fs::read_to_string(&quoted_path) else {
            println!("command_cat file_path {}: No such file or directory", &quoted_path);
            continue;
        };
        print!("{}", file_contents)
    }
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

    // borrow
    let Some(command) = args.split(' ').next() else {
        println!("command args error. args: {}", args);
        return result;
    };

    // struct 에 담기 때문에 owned
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
        // full_display 를 result 에 담아야 되기 때문에 into_owned 로 소유
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