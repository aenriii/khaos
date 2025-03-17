use deadpool_redis::{redis, Config, Connection, Runtime};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{error::Error, fs, sync::Arc};
use tokio::sync::broadcast;
use twilight_cache_inmemory::DefaultInMemoryCache;
use twilight_gateway::{Event, EventTypeFlags, Intents, Shard, ShardId, StreamExt};
use twilight_http::{request::channel::reaction::RequestReactionType, Client};
use twilight_model::{
    channel::Message,
    id::{
        marker::{ChannelMarker, MessageMarker, UserMarker},
        Id,
    },
};

const SUCCESS_REACTION: RequestReactionType = RequestReactionType::Unicode { name: "✅" };

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
        Event::GatewayHeartbeatAck => {}
        Event::GatewayHello(_) => {}
        Event::GuildCreate(_) => {}
        Event::MessageCreate(msg) => {
            parse_command(&config, http, database, &msg).await?;
        }
        Event::Ready(_) => {}
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

    let mut is_electing = false;

    let epoch = config.epoch();

    let interval = config.interval();

    let duration = config.duration();

    while let Some(msg) = shard.next_event(EventTypeFlags::all()).await {
        let Ok(event) = msg else {
            eprintln!("Failed to receive event: {msg:?}");
            continue;
        };

        let election_iter = get_election_count(
            std::time::SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            epoch,
            interval,
        );

        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let database = pool.get().await?;

        cache.update(&event);

        if !is_electing
            && current_time >= get_election_time(election_iter.await, epoch, interval).await
            && current_time <= duration
        {
            is_electing = true;
        } else {
            is_electing = false;
        }
        //We need some way for handle_event to know when to accept election votes.
        tokio::spawn(handle_event(
            config.clone(),
            Arc::clone(&http),
            database,
            event,
        ));
    }

    Ok(())
}

async fn get_election_count(time: u64, epoch: u64, interval: u64) -> u64 {
    return (time - epoch) / interval;
}

async fn get_election_time(iter: u64, epoch: u64, interval: u64) -> u64 {
    return epoch + (interval * iter);
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
                    config,
                    http,
                    database,
                    msg.channel_id,
                    msg.author.id,
                    msg.id,
                    args[1],
                )
                .await?;
            }
            _ => {}
        }
    }

    Ok(())
}

async fn parse_nomination(
    config: &KhaosControl,
    http: Arc<Client>,
    mut database: Connection,
    cid: Id<ChannelMarker>,
    author: Id<UserMarker>,
    mid: Id<MessageMarker>,
    nominee: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let members = http
        .guild_members(config.guild())
        .limit(1000)
        .await?
        .models()
        .await?;
    let nominee = if nominee.starts_with("<@") && nominee.ends_with(">") {
        let id: Id<UserMarker> = nominee[2..nominee.len() - 1].parse()?;
        members.iter().find(|&member| member.user.id == id)
    } else {
        members.iter().find(|&member| member.user.name == nominee)
    };
    if let Some(nominee) = nominee {
        if redis::cmd("SADD")
            .arg(&[format!("nominee:{}", nominee.user.id), author.to_string()])
            .query_async(&mut database)
            .await?
        {
            println!("{author} nominated {}", nominee.user.id);
            http.create_reaction(cid, mid, &SUCCESS_REACTION).await?;
        } else {
            send_message(http, "You've already nominated this user!", cid, Some(mid)).await?;
        }
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
