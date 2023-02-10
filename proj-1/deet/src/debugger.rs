use crate::debugger_command::DebuggerCommand;
use crate::inferior::Inferior;
use crate::inferior::Status;
use crate::dwarf_data:: {
    DwarfData,
    Error as DwarfError,
};
use rustyline::error::ReadlineError;
use rustyline::Editor;

pub struct Debugger {
    target: String,
    history_path: String,
    readline: Editor<()>, // Line Editor
    inferior: Option<Inferior>,
    debug_data: DwarfData,
}

impl Debugger {
    /// Initializes the debugger.
    pub fn new(target: &str) -> Debugger {
        // TODO (milestone 3): initialize the DwarfData

        let history_path = format!("{}/.deet_history", std::env::var("HOME").unwrap());
        let mut readline = Editor::<()>::new().expect("Create Editor fail");
        // Attempt to load history from ~/.deet_history if it exists
        readline.load_history(&history_path).ok();

        let debug_data = match DwarfData::from_file(target) {
            Ok(data) => data,
            Err(DwarfError::ErrorOpeningFile) => {
                println!("Counld not open file {}", target);
                std::process::exit(1);
            }
            Err(DwarfError::DwarfFormatError(e)) => {
                println!("Could not debugging symbols from {} : {:?}", target, e);
                std::process::exit(1);
            }
        }; 

        Debugger {
            target: target.to_string(),
            history_path,
            readline,
            inferior: None,
            debug_data: debug_data,
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.get_next_command() {
                DebuggerCommand::Run(args) => {
                    // kill the inferior if it is already running
                    self.kill_inferior();
                    if let Some(inferior) = Inferior::new(&self.target, &args) {
                        // Create the inferior
                        self.inferior = Some(inferior);
                        // start
                        self.continue_inferior();

                    } else {
                        println!("Error starting subprocess");
                    }
                }
                DebuggerCommand::Continue => {
                    // Continue to run
                    self.continue_inferior();
                }
                DebuggerCommand::Quit => {
                    self.kill_inferior();
                    return;
                }
                DebuggerCommand::Backtrace => {
                    if let Some(inferior) = &self.inferior {
                        inferior.backtrace(&self.debug_data);
                    } else {
                        println!("No inferior running");
                    }
                }
                _ => {
                    println!("Unknown command");
                }
            }
        }
    }

    pub fn kill_inferior(&mut self) {
        if self.inferior.is_some() {
            self.inferior.as_mut().unwrap().kill();
            self.inferior = None;
        }
    }

    pub fn continue_inferior(&mut self) {
        if let Some(inferior) = self.inferior.as_mut() {
            match inferior.cont() {
                Ok(status) => match status {
                    Status::Exited(exit_code) => {
                        println!("Inferior exited with code {}", exit_code);
                        self.inferior = None
                    }
                    Status::Signaled(signal) => {
                        println!("Inferior was killed by signal {}", signal);
                        self.kill_inferior();
                    }
                    Status::Stopped(signal, rip) => {
                        println!("Inferior stopped at rip = 0x{:x} due to signal {}", rip, signal);
                    }
                },
                Err(e) => {
                    println!("Error: {:?}", e);
                }
            }
        }
        else {
            println!("No inferior to continue");
        }
    }

    /// This function prompts the user to enter a command, and continues re-prompting until the user
    /// enters a valid command. It uses DebuggerCommand::from_tokens to do the command parsing.
    ///
    /// You don't need to read, understand, or modify this function.
    fn get_next_command(&mut self) -> DebuggerCommand {
        loop {
            // Print prompt and get next line of user input
            match self.readline.readline("(deet) ") {
                Err(ReadlineError::Interrupted) => {
                    // User pressed ctrl+c. We're going to ignore it
                    println!("Type \"quit\" to exit");
                }
                Err(ReadlineError::Eof) => {
                    // User pressed ctrl+d, which is the equivalent of "quit" for our purposes
                    return DebuggerCommand::Quit;
                }
                Err(err) => {
                    panic!("Unexpected I/O error: {:?}", err);
                }
                Ok(line) => {
                    if line.trim().len() == 0 {
                        continue;
                    }
                    self.readline.add_history_entry(line.as_str());
                    if let Err(err) = self.readline.save_history(&self.history_path) {
                        println!(
                            "Warning: failed to save history file at {}: {}",
                            self.history_path, err
                        );
                    }
                    let tokens: Vec<&str> = line.split_whitespace().collect();
                    if let Some(cmd) = DebuggerCommand::from_tokens(&tokens) {
                        return cmd;
                    } else {
                        println!("Unrecognized command.");
                    }
                }
            }
        }
    }
}
