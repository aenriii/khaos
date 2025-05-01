pub mod commands;
pub mod events;
mod helper;
pub mod routers;

use crate::di::DI;
pub use helper::*;
use std::sync::Arc;
use tokio::sync::RwLock;
use twilight_gateway::{EventTypeFlags, Shard, StreamExt};
use twilight_http::Client as HttpClient;
pub async fn run(shard: Arc<RwLock<Shard>>, _http: Arc<HttpClient>, di: DI) {
    routers::initialize_routers(di.clone());
    commands::register_commands(di.clone()).await;
    loop {
        if let Some(event) = { shard.write().await.next_event(EventTypeFlags::all()).await } {
            match event {
                Ok(ev) => {
                    let di = di.clone();
                    tokio::spawn(async move {
                        di.cache.update(&ev);
                        events::handle_event(ev, di.clone()).await;
                    });
                }
                Err(err) => {
                    eprintln!("Error receiving event: {}", err);
                }
            }
        }
    }
}
