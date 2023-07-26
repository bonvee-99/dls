#[derive(Debug)]
pub enum Command {
    Start,
    Create,
    Join,
    Send,
    List,
    Help,
    Quit,
}

impl Command {
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "start" => Some(Command::Start),
            "create" => Some(Command::Create),
            "join" => Some(Command::Join),
            "send" => Some(Command::Send),
            "list" => Some(Command::List),
            "help" => Some(Command::Help),
            "quit" => Some(Command::Quit),
            _ => None,
        }
    }

    pub fn to_string(&self) -> &'static str {
        match *self {
            Command::Start => "start",
            Command::Create => "create",
            Command::Join => "join",
            Command::Send => "send",
            Command::List => "list",
            Command::Help => "help",
            Command::Quit => "quit",
        }
    }
}
