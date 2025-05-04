use crate::di::DI;

mod nominations;

pub async fn register_interactions(di: DI) {
    let ir = di.interaction_router.get();
    if let None = ir {
        log::error!("Interaction handler not set");
        return;
    }
    let mut ir = ir.unwrap().write().await;
    ir.add_handler(&nominations::filter, &nominations::handle);

    log::info!("Interaction handlers registered");
}
