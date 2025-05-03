use crate::di::DI;

mod nominate;
mod ping;

pub async fn register_commands(di: DI) {
    if di.text_command_router.get().is_none() {
        log::error!("Text command router not initialized");
        return;
    }
    let mut p = di.text_command_router.get().unwrap().write().await;

    p.add_command(String::from("ping"), true, &ping::text);
    p.add_command(String::from("nominate"), false, &nominate::text);
    log::info!("Registered Commands");
}
