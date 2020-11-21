use std::{fmt::Write, net::ToSocketAddrs};

pub enum Command {
    Nop,
    Help,
    Exit,
    Info,
    Rm(usize),
    Add(String),
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
            .subcommand(clap::SubCommand::with_name("info"))
            .subcommand(
                clap::SubCommand::with_name("rm").about("remove tx").arg(
                    clap::Arg::with_name("index")
                        .required(true)
                        .takes_value(true)
                        .validator(|v| v.parse::<usize>().map(|_| ()).map_err(|e| e.to_string())),
                ),
            )
            .subcommand(
                clap::SubCommand::with_name("add").about("add tx").arg(
                    clap::Arg::with_name("addr")
                        .required(true)
                        .takes_value(true)
                        .validator(|v| v.to_socket_addrs().map(|_| ()).map_err(|e| e.to_string())),
                ),
            );

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
                ("rm", Some(args)) => {
                    let index = args.value_of("index").unwrap().parse::<usize>().unwrap();
                    Ok(Command::Rm(index))
                }
                ("add", Some(args)) => {
                    let addr = args.value_of("addr").unwrap();
                    Ok(Command::Add(addr.to_string()))
                }
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
