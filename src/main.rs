use std::{error::Error, fs, sync::Arc};
use twilight_cache_inmemory::DefaultInMemoryCache;
use twilight_gateway::{Event, EventTypeFlags, Intents, Shard, ShardId, StreamExt};
use twilight_http::Client;
use twilight_model::channel::Message;
use twilight_model::id::{
    marker::{ChannelMarker, GuildMarker, MessageMarker, RoleMarker, UserMarker},
    Id,
};

const BOT_ID: Id<UserMarker> = Id::new(1336849855863324763);
const GUILD_ID: Id<GuildMarker> = Id::new(1336845272541827144);
const ROLE_ID: Id<RoleMarker> = Id::new(1336846660248141975);

// TODO: Accept nominations via a command
// TODO: Create a poll every two weeks to determine the new leader of the server
// TODO: Prevent the leader from fighting the bot

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let token = fs::read_to_string(".token")?;

    let mut shard = Shard::new(
        ShardId::ONE,
        token.clone(),
        Intents::GUILDS
            | Intents::GUILD_MEMBERS
            | Intents::GUILD_MESSAGES
            | Intents::MESSAGE_CONTENT,
    );

    let http = Arc::new(Client::new(token));

    let cache = DefaultInMemoryCache::builder().build();

    send_message(
        Arc::clone(&http),
        "Bot online!",
        Id::new(1336845273263243296),
        None,
    )
    .await
    .unwrap();

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
    match event {
        Event::MessageCreate(msg) => {
            println!("MESSAGE: {}", msg.content);
            parse_command(&msg, http).await.unwrap();
        }
        _ => println!("DEBUG: {event:?}"),
    }

    Ok(())
}

async fn send_message(
    http: Arc<Client>,
    msg: &str,
    cid: Id<ChannelMarker>,
    rid: Option<Id<MessageMarker>>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if let Some(value) = rid {
        http.create_message(cid).reply(value).content(&msg).await?;
    } else {
        http.create_message(cid).content(&msg).await?;
    }

    Ok(())
}

async fn parse_command(
    msg: &Message,
    http: Arc<Client>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let msg_vec: Vec<&str> = msg.content.as_str().split_whitespace().collect();

    match msg_vec[0] {
        "!test" => {
            send_message(http, "Reply", msg.channel_id, Some(msg.id)).await?;
        }
        "!say" => {
            send_message(http, msg_vec[1], msg.channel_id, Some(msg.id)).await?;
        }
        _ => {
            println!("DEBUG: No valid command entered");
        }
    }

    Ok(())
}
