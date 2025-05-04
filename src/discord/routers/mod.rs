mod interaction;
mod slash_command;
mod text_command;
use std::{future::Future, pin::Pin};

pub use interaction::InteractionRouter;
pub use slash_command::SlashCommandRouter;
pub use text_command::{Command as TextCommand, TextCommandRouter};
use tokio::sync::RwLock;

pub type ResultFuture<T, E> = Pin<Box<dyn Future<Output = Result<T, E>> + Send>>;

/// this module holds routers for commands.
/// currently only text commands are impled
use crate::di::DI;

pub fn initialize_routers(di: DI) {
    di.text_command_router
        .set(RwLock::new(TextCommandRouter::new()));
    // di.slash_command_router
    //     .set(RwLock::new(SlashCommandRouter::new()));
    di.interaction_router
        .set(RwLock::new(InteractionRouter::new(di.clone())));
    log::info!("Registered Routers");
}
