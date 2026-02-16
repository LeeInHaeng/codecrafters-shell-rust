use rustyline::completion::{Completer, Pair};

#[derive(Clone, Debug, Default)]
pub struct MyEditorHelper {
    pub commands: Vec<String>,
}

impl MyEditorHelper {
    pub fn new(commands: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            commands: commands.into_iter().map(Into::into).collect(),
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

        // commands 리스트 중에서
        let result: Vec<Self::Candidate> = self.commands.iter()
            // line 으로 시작되는 command filter
            .filter(|command| command.starts_with(line))
            // pair 로 변경
            .map(|filtered_command| Pair
                {
                    display: filtered_command.to_string(),
                    replacement: filtered_command.to_string() 
                })
            .collect();

        Ok((0, result))
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