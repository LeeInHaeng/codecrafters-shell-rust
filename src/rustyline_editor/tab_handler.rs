use std::{io::{self, Write}, sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}}};

use rustyline::{Cmd, ConditionalEventHandler, Event, EventContext, RepeatCount};

#[derive(Debug, Default)]
pub struct MyTabHandler {
    commands: Vec<String>,
    last_was_tab: AtomicBool, // complete 가 &self(불변 참조) 여서 AtomicBool 로 (EventHandler 가 Send + Sync 여서 Cell 사용 불가능)
    filtered_commands: Mutex<Vec<String>>, // complete 가 &self(불변 참조) 여서 Mutex 로 (EventHandler 가 Send + Sync 여서 RefCell 사용 불가능)
}

impl MyTabHandler {
    pub fn new(commands: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            commands: commands.into_iter().map(Into::into).collect(),
            last_was_tab: AtomicBool::new(false),
            filtered_commands: vec![].into()
        }
    }

    pub fn get_longest_common_prefix(strs: &Vec<String>) -> String {
        // 1. 입력이 비어있으면 빈 문자열 반환
        if strs.is_empty() {
            return String::new();
        }

        // 2. 첫 번째 문자열을 접두사 기준(prefix)으로 설정
        let mut prefix = &strs[0][..];

        for s in strs.iter().skip(1) {
            // 3. 현재 prefix가 s의 시작부분(starts_with)인지 확인
            while !s.starts_with(prefix) {
                if prefix.is_empty() {
                    return String::new();
                }
                // 4. 매칭되지 않으면 prefix를 마지막에서 한 글자씩 줄임
                prefix = &prefix[..prefix.len() - 1];
            }
        }

        prefix.to_string()
    }
}

impl ConditionalEventHandler for MyTabHandler {
    fn handle(
        &self,
        _evt: &Event,
        _n: RepeatCount,
        _positive: bool,
        ctx: &EventContext,
    ) -> Option<Cmd> {
        
        let line = ctx.line();
        let filtered_commands:Vec<String>;

        // 이전에 탭을 눌렀었으면 이미 필터를 한번 했었음
        if self.last_was_tab.load(Ordering::Relaxed) {
            let filtered_commands_lock = self.filtered_commands.lock().unwrap();

            filtered_commands = filtered_commands_lock
                .iter()
                .map(|f| f.to_owned())
                .collect();

        // 이전에 탭을 누르지 않았을 경우에는 전체 commands 를 바탕으로 필터
        } else {
            filtered_commands = self.commands.iter()
                .filter(|command| command.starts_with(line))
                // owned
                .map(|f| f.to_owned())
                .collect();
        }
        self.filtered_commands.lock().unwrap().clear();

        if filtered_commands.len() <= 0 {
            // bell 울림
            print!("\x07");
            io::stdout().flush().ok();
            return Some(Cmd::Noop);

        // 실행 가능한 명령어가 정확히 1개라면 그걸로 변환해서 반환
        } else if filtered_commands.len() == 1 {
            let Some(first_filtered_command) = filtered_commands.first() else {
                return None;
            };
            let result = &first_filtered_command[line.len()..].to_string();
            let result = format!("{} ", result);

            // line 에서 추가할 부분만 뒤쪽에 추가
            return Some(Cmd::Insert(1, result));
        // 실행 가능한 명령어가 여러개일 경우
        } else {
            // 이전에 탭을 한번 눌렀을 경우
            if self.last_was_tab.load(Ordering::Relaxed) {
                self.last_was_tab.store(false, Ordering::Relaxed);
                println!("\r\n{}", filtered_commands.join("  "));

                // 이전 line 유지
                return Some(Cmd::Repaint);

            // 이전에 탭을 한번더 누르지 않았을 경우
            } else {
                // filtered_commands 의 prefix 가 모두 같으면, 그걸로 replace 해줌
                let longest_common_prefix = MyTabHandler::get_longest_common_prefix(&filtered_commands);

                // self.filtered_commands 에 추가
                let mut filtered_commands_lock = self.filtered_commands.lock().unwrap();
                filtered_commands_lock.extend(filtered_commands);

                // 일치하는 prefix 가 없거나, 현재 입력 된거랑 common prefix 가 같을 경우에
                if longest_common_prefix.is_empty() || line == longest_common_prefix {
                    // 이전 탭 눌렀음 true 로 변경
                    self.last_was_tab.store(true, Ordering::Relaxed);

                    // bell 울림
                    print!("\x07");
                    io::stdout().flush().ok();
                    return Some(Cmd::Noop);
                } else {
                    // line 에서 추가할 부분만 뒤쪽에 추가
                    let result = longest_common_prefix[line.len()..].to_string();
                    return Some(Cmd::Insert(1, result));
                }
            }
        }
    }
}

