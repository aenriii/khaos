use std::{
    ops::Deref,
    sync::{Arc, LazyLock},
};

use twilight_model::{
    application::{command::CommandType, interaction::InteractionData},
    gateway::payload::incoming::InteractionCreate,
};

use crate::{
    db::{self, models::NewNominee},
    di::DI,
    discord::InteractionHelper,
    Error,
};

pub async fn filter(args: Arc<(DI, InteractionCreate)>) -> Result<bool, Error> {
    let (_, interaction) = args.deref();
    let interaction = interaction.clone();
    match interaction.data.clone() {
        Some(InteractionData::ApplicationCommand(data)) => {
            log::trace!("Received application command interaction");
            log::trace!("Command ID: {}", data.id);

            return Ok(data.kind == CommandType::ChatInput);
        }
        _ => return Ok(false),
    }
}

pub async fn handle(args: Arc<(DI, InteractionCreate)>) -> Result<(), Error> {
    let (di, interaction) = args.deref().clone();
    match interaction.data.clone() {
        Some(InteractionData::ApplicationCommand(data)) => {
            log::trace!("Received application command interaction");
            log::trace!("Command ID: {}", data.id);

            match di.slash_command_router.get() {
                Some(router) => {
                    let router = router.read().await;
                    router.handle(interaction.0.clone(), *data.clone()).await;
                }
                None => return Err(String::from("Slash command router not initialized").into()),
            }
        }
        _ => return Err(String::from("Invalid interaction type").into()),
    }
    Ok(())
}
