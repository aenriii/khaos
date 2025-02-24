use deadpool_redis::{redis, Config, Connection, Runtime};
use std::{error::Error, fs, sync::Arc};
use twilight_cache_inmemory::DefaultInMemoryCache;
use twilight_gateway::{Event, EventTypeFlags, Intents, Shard, ShardId, StreamExt};
use twilight_http::Client;
use twilight_model::channel::Message;
use twilight_model::id::{
    marker::{ChannelMarker, MessageMarker, UserMarker},
    Id,
};

mod config;

use config::KhaosControl;

// TODO: Create a poll every two weeks to determine the new leader of the server
// TODO: Prevent the leader from fighting the bot

async fn handle_event(
    config: KhaosControl,
    http: Arc<Client>,
    database: Connection,
    event: Event,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match event {
        Event::MessageCreate(msg) => {
            println!("MESSAGE: {}", msg.content);
            parse_command(&config, http, database, &msg).await.unwrap();
        }
        _ => println!("DEBUG: {event:?}"),
    }

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = toml::from_str::<KhaosControl>(&fs::read_to_string("khaos.toml")?)?;

    let mut shard = Shard::new(
        ShardId::ONE,
        config.token().clone(),
        Intents::GUILDS
            | Intents::GUILD_MEMBERS
            | Intents::GUILD_MESSAGES
            | Intents::MESSAGE_CONTENT,
    );

    let http = Arc::new(Client::new(config.token()));

    let cache = DefaultInMemoryCache::builder().build();

    let pool = Config::from_url(config.redis()).create_pool(Some(Runtime::Tokio1))?;

    while let Some(msg) = shard.next_event(EventTypeFlags::all()).await {
        let Ok(event) = msg else {
            eprintln!("Failed to receive event: {msg:?}");
            continue;
        };

        let database = pool.get().await?;

        cache.update(&event);

        tokio::spawn(handle_event(
            config.clone(),
            Arc::clone(&http),
            database,
            event,
        ));
    }

    Ok(())
}

async fn parse_command(
    config: &KhaosControl,
    http: Arc<Client>,
    database: Connection,
    msg: &Message,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if msg.content.starts_with(config.prefix()) {
        let args: Vec<&str> = msg.content[config.prefix().len()..]
            .split_whitespace()
            .collect();

        match args[0] {
            "nominate" => {
                parse_nomination(
                    http,
                    database,
                    msg.channel_id,
                    msg.author.id,
                    msg.id,
                    args[1],
                )
                .await?;
            }
            "test" => {
                send_message(http, "Reply", msg.channel_id, Some(msg.id)).await?;
            }
            "say" => {
                send_message(http, args[1], msg.channel_id, Some(msg.id)).await?;
            }
            _ => {}
        }
    }

    Ok(())
}

async fn parse_nomination(
    http: Arc<Client>,
    mut database: Connection,
    cid: Id<ChannelMarker>,
    author: Id<UserMarker>,
    mid: Id<MessageMarker>,
    nominee: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let nid: Id<UserMarker> = nominee[2..nominee.len() - 1].parse()?;
    if redis::cmd("SADD")
        .arg(&[format!("nominee:{nid}"), author.to_string()])
        .query_async(&mut database)
        .await?
    {
        println!("LOG: {author} nominated {nid}");
        send_message(
            http,
            &format!("You've successfully nominated {nominee}!"),
            cid,
            Some(mid),
        )
        .await?;
    } else {
        send_message(http, "You've already nominated this user!", cid, Some(mid)).await?;
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
