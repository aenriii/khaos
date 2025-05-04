use crate::di::DI;

mod nominations;
mod slash_commands;
pub async fn register_interactions(di: DI) {
    let ir = di.interaction_router.get();
    if let None = ir {
        log::error!("Interaction handler not set");
        return;
    }
    let mut ir = ir.unwrap().write().await;
    ir.add_handler(&nominations::filter, &nominations::handle);
    ir.add_handler(&slash_commands::filter, &slash_commands::handle);
    log::info!("Interaction handlers registered");
}
