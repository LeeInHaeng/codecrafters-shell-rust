use std::cell::{Cell, RefCell};

use rustyline::completion::{Completer, Pair};

#[derive(Clone, Debug, Default)]
pub struct MyEditorHelper {
    commands: Vec<String>,
    last_was_tab: Cell<bool>, // complete 가 &self(불변 참조) 여서 Cell 로
    filtered_commands: RefCell<Vec<String>>, // complete 가 &self(불변 참조) 여서 RefCell 로
}

impl MyEditorHelper {
    pub fn new(commands: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            commands: commands.into_iter().map(Into::into).collect(),
            last_was_tab: Cell::new(false),
            filtered_commands: RefCell::new(vec![]),
        }
    }
}

impl rustyline::Helper for MyEditorHelper {}
impl Completer for MyEditorHelper {
    type Candidate = Pair;
    fn complete(
        &self,
        line: &str,
        _pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Self::Candidate>)> {

        // commands 리스트 중에서 line 으로 시작되는거 filter
        let filtered_commands: Vec<String>;

        // 이전에 탭을 눌렀었으면 이미 필터를 한번 했었음
        if self.last_was_tab.get() {
            // RefCell 읽기 : 불변 참조
            filtered_commands = self.filtered_commands.borrow().to_vec();

        // 이전에 탭을 누르지 않았을 경우에는 전체 commands 를 바탕으로 필터
        } else {
            filtered_commands = self.commands.iter()
                .filter(|command| command.starts_with(line))
                // owned
                .map(|f| f.to_owned())
                .collect();
        }

        if filtered_commands.len() <= 0 {
            return Ok((0, vec![]));
        // 실행 가능한 명령어가 정확히 1개라면 그걸로 변환해서 반환
        } else if filtered_commands.len() == 1 {
            let result: Vec<Pair> = filtered_commands
                .iter()
                .map(|c| Pair
                    {
                        // 변경할 문자열
                        display: c.to_string(),
                        // 변경해서 보여줄때는 공백 추가
                        replacement: format!("{} ", c)
                    })
                .collect();

            return Ok((0, result));
        // 실행 가능한 명령어가 여러개일 경우
        } else {
            let result: Vec<Pair>;

            // 이전에 탭을 한번 눌렀을 경우
            if self.last_was_tab.get() {
                self.last_was_tab.set(false);
                println!("\r\n{}", filtered_commands.join("  "));
                // RefCell 쓰기 : 가변 참조
                self.filtered_commands.borrow_mut().clear();

                // 이전 line 유지
                result = vec![Pair
                {
                    display: line.to_string(),
                    replacement: line.to_string(),
                }];

            // 이전에 탭을 한번더 누르지 않았을 경우
            } else {
                self.last_was_tab.set(true);
                // RefCell 쓰기 : 가변 참조
                self.filtered_commands.borrow_mut().extend(filtered_commands);

                // 아무것도 변환하지 않음
                result = vec![];
            }
            
            return Ok((0, result));
        }
    }
}
impl rustyline::hint::Hinter for MyEditorHelper {
    type Hint = String;
    fn hint(&self, _line: &str, _pos: usize, _: &rustyline::Context<'_>) -> Option<Self::Hint> {
        None
    }
}
impl rustyline::highlight::Highlighter for MyEditorHelper {}
impl rustyline::validate::Validator for MyEditorHelper {}