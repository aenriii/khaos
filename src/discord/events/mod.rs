use twilight_gateway::Event;

use crate::di::DI;

pub async fn handle_event(event: Event, di: DI) -> () {
    use Event::*;
    match event {
        Ready(it) => {
            log::info!(
                "[shard {}][ready] Logged in as {}!",
                it.shard.map(|sh| sh.number()).unwrap_or(0),
                it.user.name
            );
            di.current_application_id.set(it.application.id).unwrap();
        }
        MessageCreate(it) => {
            log::trace!("[message_create] Received message {}!", it.id);
            if let Some(parser) = di.text_command_router.get() {
                parser.read().await.parse_message(di.clone(), it.0).await
            } else {
                log::warn!("[message_create] No command parser found!");
            }
        }
        InteractionCreate(it) => {
            log::trace!("[interaction_create] Recieved interaction {}!", &it.id);
            if let Some(router) = di.interaction_router.get() {
                router.read().await.route(*it).await
            } else {
                log::warn!("[interaction_create] No interaction router found!");
            }
        }
        _ => {
            // log::trace!("Unhandled event: {:?}", event);
        }
    }
}
