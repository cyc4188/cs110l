pub enum DebuggerCommand {
    Quit,
    Continue,
    Run(Vec<String>),
    Backtrace,
    Breakpoint(String),
}

impl DebuggerCommand {
    pub fn from_tokens(tokens: &Vec<&str>) -> Option<DebuggerCommand> {
        match tokens[0] {
            "q" | "quit" => Some(DebuggerCommand::Quit),
            "r" | "run" => {
                let args = tokens[1..].to_vec();
                Some(DebuggerCommand::Run(
                    args.iter().map(|s| s.to_string()).collect(),
                ))
            }
            "c" | "cont" | "continue" => {
                Some(DebuggerCommand::Continue)
            }
            "bt" | "back" | "backtrace" => {
                Some(DebuggerCommand::Backtrace)
            }
            "b" | "break" | "breakpoint" => {
                if tokens.len() == 2 {
                    Some(DebuggerCommand::Breakpoint(tokens[1].to_string()))
                } else {
                    None
                }
            }
            // Default case:
            _ => None,
        }
    }
}
