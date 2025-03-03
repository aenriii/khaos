use deadpool_redis::{redis, Config, Connection, Runtime};
use std::{error::Error, fs, sync::Arc};
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

    let epoch = config.epoch();

    let interval = config.interval();

    let duration = config.duration();

    tokio::spawn(timer(Arc::clone(&http), epoch, interval, duration));

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

//Hey bread, make sure to give epoch a value compatible with SystemTime. It will probably need to
//be converted. It's currently a string only cause I wasn't sure what type it needed to be.
//~ZShamp
async fn timer(
    http: Arc<Client>,
    epoch: String,
    interval: i64,
    duration: i64,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut tick: i64 = 0;

    loop {
        if tick == interval {
            println!("DEBUG: nomination interval reached");
        } else if tick >= interval + duration {
            println!("DEBUG: Polling duration reached.");
        }

        tick += 1;
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
