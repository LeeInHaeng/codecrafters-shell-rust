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

#[derive(PartialEq, Clone)]
enum CommandOutput {
    StdOut,
    StdOutNewLine,
    File,
}

#[derive(Default)]
struct CommandExecutableResult {
    pub command: String,
    pub full_path: String,
    pub result: CommandResult,
}

#[derive(Default)]
struct RedirectionArgsBuilderResult {
    pub command_args: String,
    pub output: String,
    pub result: CommandResult
}

fn main() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        // owned
        let mut input = String::new();

        io::stdin().read_line(&mut input).unwrap();
        
        let input_command = input.trim();

        // command 는 command_args_builder 함수를 태워야 되기 때문에 다 owned 로 한다.
        let mut command = String::new();
        // command_args 는 단순 slice 로 가능해서 borrowed 로 한다.
        let command_args;

        // command 가 쿼터로 묶여 있으면
        if input_command.starts_with('\'') || input_command.starts_with('\"') {
            let split_pattern;
            if input_command.starts_with('\'') {
                split_pattern = '\'';
            } else {
                split_pattern = '\"';
            }

            // 마지막 쿼터까지 한번 파싱
            let Some(command_filter) = input_command.strip_prefix(split_pattern) else {
                println!("command quotes invalid");
                continue;
            };
            let Some(command_quote_end_idx) = command_filter.find(split_pattern) else {
                println!("command single quotes invalid");
                continue;
            };

            // command_filter 섀도잉
            let command_filter = &command_filter[..command_quote_end_idx];

            // 기존에 묶여있던 쿼터로 다시 한번 묶어줌
            let mut command_args_builder_param = String::new();
            if split_pattern == '\'' {
                command_args_builder_param.push('\'');
            } else {
                command_args_builder_param.push('"');
            }
            command_args_builder_param.push_str(command_filter);
            if split_pattern == '\'' {
                command_args_builder_param.push('\'');
            } else {
                command_args_builder_param.push('"');
            }

            command = special_char_args_builder(&command_args_builder_param);

            let Some(command_args_filter) = input_command.strip_prefix(&command_args_builder_param) else {
                println!("command single quotes invalid");
                continue;
            };
            command_args = command_args_filter.trim();

        // 아무것도 묶여있지 않으면 공백으로 커맨드, 커맨드 파라미터 분리
        } else if input_command.contains(" ") {
            if let Some((cmd, rest)) = input_command.split_once(' ') {
                command = cmd.to_string();
                command_args = rest.trim();
            } else {
                command_args = "";
            }
        // 공백 조차 없으면 파라미터가 없는거
        } else {
            command = input_command.to_string();
            command_args = "";
        }

        // command 섀도잉
        let command = &command[..];

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
                    // 기본 cat 은 /tmp 하위로 기본 셋팅 해둔다.
                    "cat" => command_cat(&command_args),
                    "ls" => command_ls(&command_args),
                    _ => command_execute(command, &command_args)
                };
            }
        };
    }
}

fn command_output(enum_output: CommandOutput, args: &str, writer_output: &str) {
    if enum_output == CommandOutput::StdOut {
        print!("{}", args);
        return;
    }

    if enum_output == CommandOutput::StdOutNewLine {
        println!("{}", args);
        return;
    }

    if enum_output == CommandOutput::File {
        let _ = fs::write(writer_output, args);
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
fn special_char_args_builder(args: &str) -> String {
    let mut result = String::with_capacity(args.len());
    let mut is_quote_start = false;
    let mut is_double_quote = false;
    let mut is_ignore_next = false;
    let mut is_ignore_backslash = false;

    for (idx, char) in args.char_indices() {
        let mut before_char = '\0';
        if 0 < idx {
            before_char = args.as_bytes()[idx - 1] as char;
        }

        // 백슬래쉬에 대해 다음 나올 문자열을 무시할지 판단
        if char == '\\' && false == is_ignore_next {
            // 쿼터로 안묶여 있는 경우
            if  false == is_quote_start {
                is_ignore_next = true;
                continue;
            }
            // 더블 쿼터로 묶여 있을 경우
            if true == is_quote_start && true == is_double_quote {
                is_ignore_next = true;
                continue;
            }
        }

        if is_ignore_next {
            result.push(char);
            is_ignore_next = false;
            // 무시된 문자열이 백슬래쉬인 경우 해당 백슬래쉬 효과도 다음에 무시
            if char == '\\' {
                is_ignore_backslash = true;
            }
            continue;
        }

        if char == '\'' || char == '\"' {
            // 이전 문자가 blackslash 일 경우 \'\ 가 아니면 현재꺼를 담고 무시
            if before_char == '\\' && char == '\"' && false == is_ignore_backslash {
                result.push(char);
                continue;
            }

            if is_quote_start {
                // double quotes 로 묶인거면 single quotes 는 string 에 담고 무시
                if is_double_quote && char == '\'' {
                    result.push(char);
                    continue;
                }
                // single quotes 로 묶인거면 double quotes 는 string 에 담고 무시
                if false == is_double_quote && char == '\"' {
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

fn is_redirection_args(args: &str) -> bool {
    if args.contains(">") {
        return true;
    }
    false
}

fn redirection_args_builder(args: &str) -> RedirectionArgsBuilderResult {
    let mut result = RedirectionArgsBuilderResult::default();

    let mut splited_redirection_str = ">";
    if args.contains("1>") {
        splited_redirection_str = "1>";
    }

    let mut splited_redirection_args:Vec<&str> = args.split(splited_redirection_str).collect();
    let Some(output) = splited_redirection_args.pop() else {
        println!("command args output error 1. args: {}", args);
        return result;
    };

    result.output = output.trim().to_string();

    let Some(command_args) = splited_redirection_args.first() else {
        println!("command args output error 2. args: {}", args);
        return result;
    };

    result.command_args = command_args.to_string();
    result.result = CommandResult::Success;

    result
}

fn command_echo(args: &str) {
    let echo_args_builder;
    let command_output_enum;
    let writer_output;

    if is_redirection_args(args) {
        let redirection_args_builder_result: RedirectionArgsBuilderResult = redirection_args_builder(args);
        if redirection_args_builder_result.result != CommandResult::Success {
            return;
        }

        echo_args_builder = redirection_args_builder_result.command_args;
        command_output_enum = CommandOutput::File;
        writer_output = redirection_args_builder_result.output;
    } else {
        echo_args_builder = args.to_string();
        command_output_enum = CommandOutput::StdOutNewLine;
        writer_output = "".to_string();
    }

    let echo_args_builder = special_char_args_builder(&echo_args_builder);
    if echo_args_builder.is_empty() {
        return;
    }

    command_output(command_output_enum, &echo_args_builder, &writer_output);
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
    command_execute("ls", args);

    /*
    let cat_args_builder;
    let command_output_enum;
    let writer_output;
    let is_redirection_args = is_redirection_args(args);

    if is_redirection_args {
        let redirection_args_builder_result: RedirectionArgsBuilderResult = redirection_args_builder(args);
        if redirection_args_builder_result.result != CommandResult::Success {
            return;
        }

        cat_args_builder = redirection_args_builder_result.command_args;
        command_output_enum = CommandOutput::File;
        writer_output = redirection_args_builder_result.output;
    } else {
        cat_args_builder = args.to_string();
        command_output_enum = CommandOutput::StdOut;
        writer_output = "".to_string();
    }

    let cat_args_builder = special_char_args_builder(&cat_args_builder);
    let file_path_args = split_by_anchor_segments(&cat_args_builder, "/tmp");

    if file_path_args.len() == 0 {
        let Ok(file_contents) = fs::read_to_string(&cat_args_builder) else {
            command_output(CommandOutput::StdOutNewLine, &format!("cat: {}: No such file or directory", &cat_args_builder), "");
            if is_redirection_args {
                command_output(CommandOutput::File, "", &writer_output);
            }
            return;
        };
        command_output(command_output_enum, &file_contents, &writer_output);
    } else {
        for file_path in file_path_args {
            let Ok(file_contents) = fs::read_to_string(&file_path) else {
                command_output(CommandOutput::StdOutNewLine, &format!("cat: {}: No such file or directory", &file_path), "");
                if is_redirection_args {
                    command_output(CommandOutput::File, "", &writer_output);
                }
                continue;
            };
            command_output(command_output_enum.clone(), &file_contents, &writer_output);
        }
    }
     */
}

fn strip_wrapping_quotes(s: &str) -> &str {
    let s = s.trim();
    if s.len() >= 2 {
        let b = s.as_bytes();
        if (b[0] == b'"' && b[b.len() - 1] == b'"') || (b[0] == b'\'' && b[b.len() - 1] == b'\'') {
            return &s[1..s.len() - 1];
        }
    }
    s
}

/// Split into segments that start with `anchor` (e.g. "/tmp/fox/").
/// Segment ends right before the next anchor occurrence (at a boundary).
fn split_by_anchor_segments(input: &str, anchor: &str) -> Vec<String> {
    let input = strip_wrapping_quotes(input);
    let bytes = input.as_bytes();
    let a = anchor.as_bytes();

    let mut out = Vec::new();
    let mut seg_start: Option<usize> = None;

    let mut i = 0usize;
    while i + a.len() <= bytes.len() {
        let is_anchor = bytes[i..].starts_with(a);
        let boundary_ok = i == 0 || bytes[i - 1].is_ascii_whitespace(); // 필요하면 더 확장 가능

        if is_anchor && boundary_ok {
            if let Some(s) = seg_start {
                let seg = input[s..i].trim();
                if !seg.is_empty() {
                    out.push(seg.to_string());
                }
            }
            seg_start = Some(i);
        }
        i += 1;
    }

    if let Some(s) = seg_start {
        let seg = input[s..].trim();
        if !seg.is_empty() {
            out.push(seg.to_string());
        }
    }

    out
}

fn command_ls(args: &str) {
    command_execute("ls", args);
}

fn command_execute(command: &str, command_args: &str) {
    let check_command_executable_result = check_command_executable(command);
    if CommandResult::Success != check_command_executable_result.result {
        return;
    }

    let command_execute_args_builder;
    let command_output_enum;
    let writer_output;

    if is_redirection_args(command_args) {
        let redirection_args_builder_result: RedirectionArgsBuilderResult = redirection_args_builder(command_args);
        if redirection_args_builder_result.result != CommandResult::Success {
            return;
        }

        command_execute_args_builder = redirection_args_builder_result.command_args;
        command_output_enum = CommandOutput::File;
        writer_output = redirection_args_builder_result.output;
    } else {
        command_execute_args_builder = command_args.to_string();
        command_output_enum = CommandOutput::StdOut;
        writer_output = "".to_string();
    }

    println!("{:?}", command_execute_args_builder.split_whitespace().collect::<Vec<&str>>());

    // execute command
    match Command::new(check_command_executable_result.command).args(command_execute_args_builder.split_whitespace()).output() {
        Ok(output) => {
            command_output(command_output_enum, str::from_utf8(&output.stdout).unwrap(), &writer_output);
        },
        Err(e) => {
            println!("{}", e);
        }
    }
}

fn check_command_executable(command: &str) -> CommandExecutableResult {
    let mut result: CommandExecutableResult = CommandExecutableResult::default();

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