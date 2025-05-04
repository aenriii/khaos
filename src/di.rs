use std::sync::Arc;

use tokio::sync::{OnceCell, RwLock};
use twilight_cache_inmemory::DefaultInMemoryCache;
use twilight_gateway::Shard;
use twilight_model::id::{marker::ApplicationMarker, Id};

use crate::{
    config::Config,
    db::DbPool,
    discord::routers::{InteractionRouter, SlashCommandRouter, TextCommandRouter},
};

/// Dependency Injection container for the bot.
/// All references are Arc or otherwise impl Clone
#[derive(Debug, Clone)]
pub struct DI {
    pub db: Arc<DbPool>,
    pub discord_gateway: Arc<RwLock<Shard>>,
    pub discord_http: Arc<twilight_http::Client>,
    pub text_command_router: Arc<OnceCell<RwLock<TextCommandRouter>>>,
    pub interaction_router: Arc<OnceCell<RwLock<InteractionRouter>>>,
    pub slash_command_router: Arc<OnceCell<RwLock<SlashCommandRouter>>>,
    pub config: Arc<Config>,
    pub cache: Arc<DefaultInMemoryCache>,

    pub current_application_id: Arc<OnceCell<Id<ApplicationMarker>>>,
}
unsafe impl Send for DI {}
unsafe impl Sync for DI {}
