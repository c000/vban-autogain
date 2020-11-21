use std::fmt::Write;

pub enum Command {
    Nop,
    Help,
    Exit,
    Info,
    Error(String),
}

pub struct App<'a> {
    app: clap::App<'a, 'a>,
    help: std::vec::Vec<u8>,
}

impl App<'_> {
    pub fn new() -> Self {
        let app = clap::App::new("")
            .subcommand(clap::SubCommand::with_name("help"))
            .subcommand(clap::SubCommand::with_name("exit"))
            .subcommand(clap::SubCommand::with_name("info"));

        let help = get_help_vec(&app);

        return App { app, help };
    }

    pub fn parse_command(&mut self, line: &str) -> std::io::Result<Command> {
        let words = shell_words::split(line).expect("Failed to split repl words");

        let matches = self
            .app
            .get_matches_from_safe_borrow(std::iter::once(String::new()).chain(words));

        match matches {
            Err(e) => {
                let mut message = e.message;
                writeln!(message).unwrap();
                Ok(Command::Error(message))
            }
            Ok(m) => match m.subcommand() {
                ("help", _) => Ok(Command::Help),
                ("exit", _) => Ok(Command::Exit),
                ("info", _) => Ok(Command::Info),
                _ => Ok(Command::Nop),
            },
        }
    }

    pub fn help(&self) -> &[u8] {
        self.help.as_slice()
    }
}

fn get_help_vec<'a, 'b>(app: &clap::App<'a, 'b>) -> std::vec::Vec<u8> {
    let mut b = Vec::new();
    app.write_help(&mut b).unwrap();
    b.push(b'\n');
    b
}
