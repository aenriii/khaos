use twilight_interactions::command::CreateCommand;
use twilight_model::application::command::Command;

use crate::di::DI;

mod nominate;
mod ping;
mod test;

pub async fn register_commands(di: DI) {
    if di.text_command_router.get().is_none() {
        log::error!("Text command router not initialized");
        return;
    }
    let mut p = di.text_command_router.get().unwrap().write().await;

    p.add_command(String::from("ping"), true, &ping::text);
    p.add_command(String::from("nominate"), false, &nominate::text);
    p.add_command(String::from("test"), false, &test::text);

    if di.slash_command_router.get().is_none() {
        log::error!("Slash command router not initialized");
        return;
    }
    let mut p = di.slash_command_router.get().unwrap().write().await;

    p.add_command(String::from("ping"), &ping::PingCommand::handle);
    p.add_command(String::from("nominate"), &nominate::NominateCommand::handle);

    log::info!("Registered Commands");
}

pub fn slash_commands_list() -> Vec<Command> {
    return vec![
        ping::PingCommand::create_command().into(),
        nominate::NominateCommand::create_command().into(),
    ];
}
