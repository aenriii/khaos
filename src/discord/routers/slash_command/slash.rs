use std::{fmt, future::Future};

use async_trait::async_trait;
use twilight_interactions::command::CreateCommand;
use twilight_model::application::interaction::{application_command::CommandData, Interaction};

use crate::{di::DI, discord::routers::ResultFuture, Error};

pub trait SlashFn<T>: 'static {
    fn call(&self, di: DI, interaction: Interaction, data: CommandData) -> ResultFuture<T, Error>;
}

pub type SlashHandlerFn = &'static (dyn SlashFn<()> + Send + Sync);

impl<F, G, T> SlashFn<T> for F
where
    F: Fn(DI, Interaction, CommandData) -> G + 'static,
    G: Future<Output = Result<T, Error>> + Send + 'static,
{
    fn call(&self, di: DI, interaction: Interaction, data: CommandData) -> ResultFuture<T, Error> {
        let fut = (self)(di, interaction, data);
        Box::pin(async move { fut.await })
    }
}

pub struct SlashCommand {
    pub name: String,
    handler: SlashHandlerFn,
}

impl SlashCommand {
    pub fn new(name: String, handler: SlashHandlerFn) -> Self {
        Self { name, handler }
    }
    pub async fn handle(
        &self,
        di: DI,
        interaction: Interaction,
        data: CommandData,
    ) -> Result<(), Error> {
        let fut = self.handler.call(di, interaction.clone(), data);
        let res = fut.await;
        if let Err(_) = res {
            log::error!("Error executing slash command: {}", interaction.id);
        }
        return res;
    }
}

impl fmt::Debug for SlashCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SlashCommand")
            .field("name", &self.name)
            .finish()
    }
}
