use std::sync::Arc;

use twilight_interactions::command::{CommandModel, CreateCommand};
use twilight_model::{
    application::interaction::{application_command::CommandData, Interaction},
    channel::Message,
};

use crate::{
    di::DI,
    discord::{DiscordHelper, InteractionHelper},
};

pub async fn text(args: Arc<(Message, Vec<String>, DI)>) -> Result<(), crate::Error> {
    let (message, _, di) = args.as_ref();
    DiscordHelper::reply(di.clone(), message.clone(), "Pong!").await;
    Ok(())
}

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "ping", desc = "hello world!")]
pub struct PingCommand;

impl PingCommand {
    pub async fn handle(
        di: DI,
        interaction: Interaction,
        data: CommandData,
    ) -> Result<(), crate::Error> {
        InteractionHelper::update_response(di.clone(), interaction, "Pong!".to_string(), vec![])
            .await;
        Ok(())
    }
}
