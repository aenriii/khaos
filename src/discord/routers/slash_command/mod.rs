use slash::{SlashCommand, SlashHandlerFn};
use twilight_model::application::interaction::{application_command::CommandData, Interaction};

use crate::{di::DI, Error};

mod slash;
#[derive(Debug)]
pub struct SlashCommandRouter {
    commands: Vec<SlashCommand>,
    di: DI,
}

impl SlashCommandRouter {
    pub fn new(di: DI) -> Self {
        Self {
            commands: Vec::new(),
            di,
        }
    }
    pub async fn handle(&self, interaction: Interaction, data: CommandData) -> Result<(), Error> {
        let command_name = data.clone().name;
        for command in &self.commands {
            if command.name == command_name {
                command.handle(self.di.clone(), interaction, data).await?;
                break;
            }
        }
        Err("Command not found".to_string().into())
    }
    pub fn add_command(&mut self, name: String, handler: SlashHandlerFn) {
        self.commands.push(SlashCommand::new(name, handler));
    }
}
