use std::{error::Error, fs, sync::Arc};
use twilight_cache_inmemory::DefaultInMemoryCache;
use twilight_gateway::{Event, EventTypeFlags, Intents, Shard, ShardId, StreamExt};
use twilight_http::Client;
use twilight_model::id::{
    marker::{GuildMarker, RoleMarker},
    Id,
};

const GUILD_ID: Id<GuildMarker> = Id::new(1330311796480938056);
const ROLE_ID: Id<RoleMarker> = Id::new(1330314630567825429);

// TODO: Accept nominations via a command
// TODO: Create a poll every two weeks to determine the new leader of the server
// TODO: Prevent the leader from fighting the bot

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let token = fs::read_to_string(".token")?;

    let mut shard = Shard::new(
        ShardId::ONE,
        token.clone(),
        Intents::GUILDS | Intents::GUILD_MEMBERS,
    );

    let http = Arc::new(Client::new(token));

    let cache = DefaultInMemoryCache::builder().build();

    while let Some(msg) = shard.next_event(EventTypeFlags::all()).await {
        let Ok(event) = msg else {
            eprintln!("Failed to receive event: {msg:?}");
            continue;
        };

        cache.update(&event);

        tokio::spawn(handle_event(event, Arc::clone(&http)));
    }

    Ok(())
}

async fn handle_event(event: Event, http: Arc<Client>) -> Result<(), Box<dyn Error + Send + Sync>> {
    println!("DEBUG: {event:?}");

    Ok(())
}
