mod slash_command;
mod text_command;

pub use slash_command::SlashCommandRouter;
pub use text_command::{Command as TextCommand, TextCommandRouter};
use tokio::sync::RwLock;

/// this module holds routers for commands.
/// currently only text commands are impled
use crate::di::DI;

pub fn initialize_routers(di: DI) {
    di.text_command_router
        .set(RwLock::new(TextCommandRouter::new()));
    // di.slash_command_router
    //     .set(RwLock::new(SlashCommandRouter::new()));
    log::info!("Registered Routers");
}
