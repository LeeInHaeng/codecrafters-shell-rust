use std::{borrow::Cow, env, fs::{self, OpenOptions}, path::Path, process::Command};
#[allow(unused_imports)]
use std::io::{self, Write};

use is_executable::IsExecutable;
use rustyline::{Editor, error::ReadlineError};

mod rustyline_editor;
use crate::rustyline_editor::editor_helper::MyEditorHelper;


const COMMAND: [&str; 5]= ["exit", "echo", "type", "pwd", "cd"];
const COMMAND_PATH: [&str; 4] = ["cat", "ls", "cat.exe", "ls.exe"];

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
    FileAppend,
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
    pub redirect: String,
    pub result: CommandResult
}

fn main() {
    let mut readline_editor: Editor<MyEditorHelper, _> = Editor::new().expect("rustyline editor fail");

    let my_editor_helper = MyEditorHelper::new(get_all_executable_command());
    readline_editor.set_helper(Some(my_editor_helper));

    loop {
        let readline = readline_editor.readline("$ ");
        
        let input_command: String = match readline {
            Ok(line) => {
                line
            }
            Err(ReadlineError::Interrupted) => {
                println!("Ctrl-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("Ctrl-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        };

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

            let command_args_builder_result = special_char_args_builder(&command_args_builder_param);
            command = command_args_builder_result.join(" ");

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
                    // cat 과 ls 는 구현이 아닌 외부에 이미 있는 command 를 사용 하게끔 한다
                    "cat" => command_cat(&command_args),
                    "ls" => command_ls(&command_args),
                    _ => command_execute(command, &command_args)
                };
            }
        };
    }
}

fn get_all_executable_command() -> Vec<String> {
    let mut result: Vec<String> = COMMAND.into_iter().map(String::from).collect();

    // get PATH
    let Some(paths) = env::var_os("PATH") else {
        println!("var os PATH not found");
        return result;
    };

    for path in env::split_paths(&paths) {
        // 환경변수 하위의 전체 dir
        let Ok(read_dir) = fs::read_dir(&path) else {
            continue;
        };

        for dir in read_dir {
            let sub_path = match dir {
                Ok(v) => v.path(),
                Err(_) => continue
            };
            let full = path.join(&sub_path);

            // Check if a file with the command name exists.
            if fs::metadata(&full).is_err() {
                continue;
            }

            if false == full.is_executable() {
                continue;
            }

            let Some(execute_file_name) = sub_path.file_name() else {
                continue;
            };
            let Some(execute_file_name) = execute_file_name.to_str() else {
                continue;
            };

            result.push(execute_file_name.to_string());
        }

    }

    result
}

fn command_output(enum_output: CommandOutput, args: &str, writer_output: &str) {
    if enum_output == CommandOutput::StdOut && false == args.is_empty() {
        print!("{}", args);
        return;
    }

    if enum_output == CommandOutput::StdOutNewLine && false == args.is_empty() {
        println!("{}", args);
        return;
    }

    if enum_output == CommandOutput::File {
        match fs::write(writer_output, args) {
            Ok(_) => {},
            Err(e) => println!("file write error. path: {}, e: {}", writer_output, e)
        };
        return;
    }

    if enum_output == CommandOutput::FileAppend {
        match OpenOptions::new()
        .append(true)
        .create(true)
        .open(writer_output) {
            Ok(mut f) => {
                match write!(f, "{}", args) {
                    Ok(_) => {},
                    Err(e) => println!("file contents write error. path: {}, e: {}", writer_output, e)
                }
            },
            Err(e) => println!("file write error. path: {}, e: {}", writer_output, e)
        };
        return;
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

// "example\"insidequotes"world\" : example"insidequotesworld"
// \'\"world example\"\' : '"world example"'
// "mixed\"quote'world'\\" : mixed"quote'world'\
// "test  world"  "shell""script" : test  world shellscript
// "script\"insidequotes"example\" : script"insidequotesexample"
// /tmp/dog/"number 41" /tmp/dog/"doublequote \" 22" /tmp/dog/"backslash \\ 82" : [0] /tmp/dog/number 41 , [1] /tmp/dog/doublequote " 22 , [2] /tmp/dog/backslash \ 82
fn special_char_args_builder(args: &str) -> Vec<String> {
    let mut result = vec![];

    let mut result_tmp = String::with_capacity(args.len());
    let mut is_quote_start = false;
    let mut is_double_quote = false;
    let mut is_ignore_next = false;
    let mut is_ignore_backslash = false;
    let mut is_ignore_result_push = false;

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
            result_tmp.push(char);
            is_ignore_next = false;
            // 무시된 문자열이 백슬래쉬인 경우 해당 백슬래쉬 효과도 다음에 무시
            if char == '\\' {
                is_ignore_backslash = true;
            }

            if char == '\'' || char == '\"' {
                if is_ignore_backslash {
                    is_ignore_result_push = false;
                } else {
                    is_ignore_result_push = true;
                }
            }
            continue;
        }

        if char == '\'' || char == '\"' {
            // 이전 문자가 blackslash 일 경우 \'\ 가 아니면 현재꺼를 담고 무시
            if before_char == '\\' && char == '\"' && false == is_ignore_backslash {
                result_tmp.push(char);
                continue;
            }

            if is_quote_start {
                // double quotes 로 묶인거면 single quotes 는 string 에 담고 무시
                if is_double_quote && char == '\'' {
                    result_tmp.push(char);
                    continue;
                }
                // single quotes 로 묶인거면 double quotes 는 string 에 담고 무시
                if false == is_double_quote && char == '\"' {
                    result_tmp.push(char);
                    continue;
                }

                is_quote_start = false;
                is_double_quote = false;
                is_ignore_result_push = false;
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
                result_tmp.push(char);
                continue;
            // 쿼터로 묶인 단계가 아닐 경우
            } else {
                // 현재 char 이 공백인 경우
                if char == ' ' {
                    // 이전 인덱스의 char 가 공백이면 중복 공백 제거를 위해 pass
                    if before_char == ' ' {
                        continue;
                    }

                    // 쿼터로 묶여 있는게 아니라면 담고 pass
                    if false == is_quote_start && is_ignore_result_push {
                        result_tmp.push(char);
                        continue;
                    }

                    // 공백 기준 구분으로 result 에 push
                    if result_tmp.trim().is_empty() {
                        continue;
                    }

                    result.push(result_tmp);
                    result_tmp = String::with_capacity(args.len());
                    continue;
                // string push
                } else {
                    result_tmp.push(char);
                    continue;
                }
            }
        }
    }

    if false == result_tmp.is_empty() {
        result.push(result_tmp);
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

    let splited_redirection_str;

    if args.contains("1>>") {
        splited_redirection_str = "1>>";
        result.redirect = "1>>".to_string();
    } else if args.contains("2>>") {
        splited_redirection_str = "2>>";
        result.redirect = "2>>".to_string();
    } else if args.contains(">>") {
        splited_redirection_str = ">>";
        result.redirect = ">>".to_string();
    } else if args.contains("1>") {
        splited_redirection_str = "1>";
        result.redirect = "1>".to_string();
    } else if args.contains("2>") {
        splited_redirection_str = "2>";
        result.redirect = "2>".to_string();
    } else {
        splited_redirection_str = ">";
        result.redirect = ">".to_string();
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

    let mut command_args = command_args.trim().to_string();
    command_args.push_str("\r\n");
    result.command_args = command_args;

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
        writer_output = redirection_args_builder_result.output;

        if redirection_args_builder_result.redirect == "2>" || redirection_args_builder_result.redirect == "2>>" {
            command_output_enum = CommandOutput::StdOut;
            // 파일 없더라도 생성 필요
            command_output(CommandOutput::File, "", &writer_output);
        } else if redirection_args_builder_result.redirect == ">>" || redirection_args_builder_result.redirect == "1>>" {
            command_output_enum = CommandOutput::FileAppend;
        } else {
            command_output_enum = CommandOutput::File;
        }
    } else {
        echo_args_builder = args.to_string();
        command_output_enum = CommandOutput::StdOutNewLine;
        writer_output = "".to_string();
    }

    let echo_args_builder = special_char_args_builder(&echo_args_builder);
    if echo_args_builder.is_empty() {
        return;
    }

    let echo_args_builder = echo_args_builder.join(" ");
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
    command_execute("cat", args);
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
    let mut command_output_enum;
    let writer_output;
    let mut is_error_redirect = false;

    if is_redirection_args(command_args) {
        let redirection_args_builder_result: RedirectionArgsBuilderResult = redirection_args_builder(command_args);
        if redirection_args_builder_result.result != CommandResult::Success {
            return;
        }

        command_execute_args_builder = redirection_args_builder_result.command_args;
        if redirection_args_builder_result.redirect == "1>>" 
            || redirection_args_builder_result.redirect == ">>"
            || redirection_args_builder_result.redirect == "2>>" {
            command_output_enum = CommandOutput::FileAppend
        } else {
            command_output_enum = CommandOutput::File;
        }
        writer_output = redirection_args_builder_result.output;

        if redirection_args_builder_result.redirect == "2>" || redirection_args_builder_result.redirect == "2>>" {
            is_error_redirect = true;
        }

        // redirection 은 내용 상관 없이 일단 파일 생성
        command_output(command_output_enum.clone(), "", &writer_output);
    } else {
        command_execute_args_builder = command_args.to_string();
        command_output_enum = CommandOutput::StdOut;
        writer_output = "".to_string();
    }

    let command_execute_args_builder = command_execute_args_builder.trim();
    let command_args_vec = special_char_args_builder(command_execute_args_builder);

    let mut valid_command_args:Vec<String> = vec![];
    let mut error_messages:Vec<String> = vec![];

    for command_arg in command_args_vec {
        let command_arg = command_arg.trim();

        if command_arg.is_empty() {
            continue;
        }

        // 하이푼이 붙은 옵션이면 무시, 옵션이 아니면 경로 존재 하는지 확인
        if command_arg.starts_with("-") {
            valid_command_args.push(command_arg.to_string());
            continue;
        }

        // 명확하게 path 가 들어오는 command 인 경우 있는 path 인지 확인
        if COMMAND_PATH.contains(&command) {
            let path = Path::new(&command_arg);
            if false == path.exists() {
                let error_message = format!("{}: {}: No such file or directory\r\n", check_command_executable_result.command, command_arg);
                error_messages.push(error_message);
                continue;
            }
        }

        valid_command_args.push(command_arg.to_string());
    }

    let error_message = &error_messages.join("");
    // 에러가 있는 경우
    if false == error_message.is_empty() {
        // 2> 혹은 2>> 인 경우 에러 내용을 기록
        if is_error_redirect {
            command_output(command_output_enum.clone(), error_message, &writer_output);
            // 성공하는 args 도 섞여 있을 수 있기 때문에 표준 출력을 위해 StdOutNewLine 으로 변경
            command_output_enum = CommandOutput::StdOut;
        // 2> 와 2>> 가 아닐 경우 에러를 표준 출력
        } else {
            command_output(CommandOutput::StdOut, error_message, &writer_output);
        }
    }

    // valid_command_args 요소중 "-" 로 시작하는 옵션 외에 있을 경우만 실행
    let is_command_execute = valid_command_args.iter().any(|v| false == v.starts_with("-"));

    // execute command
    if is_command_execute {
        match Command::new(check_command_executable_result.command).args(valid_command_args).output() {
            Ok(output) => {
                command_output(command_output_enum, str::from_utf8(&output.stdout).unwrap(), &writer_output);
            },
            Err(e) => {
                println!("{}", e);
            }
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