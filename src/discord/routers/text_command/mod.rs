mod command;
pub use command::*;

use crate::di::DI;
use twilight_model::channel::Message;

#[derive(Debug)]
pub struct TextCommandRouter {
    pub commands: Vec<Command>,
}
impl TextCommandRouter {
    pub fn new() -> Self {
        Self { commands: vec![] }
    }
    pub fn add_command(&mut self, command_name: String, callback: CommandHandler) {
        self.commands.push(Command::new(command_name, callback))
    }

    pub async fn parse_message(&self, di: DI, message: Message) {
        let content = message.content.clone();

        if !content.starts_with(&di.config.prefix) {
            return;
        }
        let args = content.split_whitespace().collect::<Vec<&str>>();
        let command_name = args[0]
            .to_lowercase()
            .trim_start_matches(&di.config.prefix)
            .to_string();
        let args = args[1..]
            .to_vec()
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        let di = di.clone();
        log::trace!("Looking for a command {}", command_name);
        // find the command in the list of commands
        if let Some(command) = self.commands.iter().find(|c| c.name == command_name) {
            log::trace!("Found command {}", command_name);
            command.execute(message, args, di);
        } else {
            log::warn!("Command not found: {}", command_name);
        }
    }
}
