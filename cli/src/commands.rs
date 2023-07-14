pub enum Command {
    Start,
    CreateRoom,
    JoinRoom,
    Send,
    List,
    Help,
    Quit,
}

impl Command {
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "start" => Some(Command::Start),
            "create room" => Some(Command::CreateRoom),
            "join room" => Some(Command::JoinRoom),
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
            Command::CreateRoom => "create room",
            Command::JoinRoom => "join room",
            Command::Send => "send",
            Command::List => "list",
            Command::Help => "help",
            Command::Quit => "quit",
        }
    }
}