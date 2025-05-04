#![feature(
    unboxed_closures,
    async_fn_traits,
    impl_trait_in_assoc_type,
    let_chains
)]
#![allow(unused)]
pub(crate) mod config;
pub(crate) mod db;
pub(crate) mod di;
pub(crate) mod discord;
use dotenvy::{dotenv, from_path};
use std::sync::Arc;
use tokio::sync::{OnceCell, RwLock};
use twilight_cache_inmemory::DefaultInMemoryCache;
use twilight_gateway::{Intents, Shard, ShardId};
use twilight_http::Client as HttpClient;
#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    {
        println!("Loading dev.env!");
        dotenv().ok();
        from_path("dev.env").ok();
    }
    #[cfg(not(debug_assertions))]
    {
        dotenv().ok();
    }

    // logger
    env_logger::init();

    // start discord things
    let discord_token = dotenvy::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set");

    let shard = Arc::new(RwLock::new(Shard::new(
        ShardId::ONE,
        discord_token.clone(),
        Intents::GUILD_MESSAGES
            | Intents::GUILD_MEMBERS
            | Intents::MESSAGE_CONTENT
            | Intents::DIRECT_MESSAGES
            | Intents::GUILD_MESSAGE_POLLS,
    )));
    let http = Arc::new(HttpClient::new(discord_token));

    // db and config
    let connection = match db::establish_pool() {
        Ok(pool) => Arc::new(pool),
        Err(err) => {
            eprintln!("Failed to establish database connection: {}", err);
            std::process::exit(1);
        }
    };

    let config = config::load_config(Some("config.toml"));

    let cache = Arc::new(DefaultInMemoryCache::builder().build());

    let di = di::DI {
        db: connection.clone(),
        discord_gateway: shard.clone(),
        discord_http: http.clone(),
        text_command_router: Arc::new(OnceCell::new()),
        interaction_router: Arc::new(OnceCell::new()),
        config: Arc::new(config),
        cache: cache.clone(),

        current_application_id: Arc::new(OnceCell::new()),
    };
    discord::run(shard, http, di).await;
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
