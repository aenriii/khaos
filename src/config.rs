use serde::Deserialize;
use twilight_model::id::{
    marker::{GuildMarker, RoleMarker},
    Id,
};

// I did a funny. ~ahill
#[derive(Clone, Deserialize)]
pub(crate) struct KhaosControl {
    guild: Id<GuildMarker>,
    prefix: String,
    redis: String,
    role: Id<RoleMarker>,
    token: String,
}

impl KhaosControl {
    pub fn guild(&self) -> Id<GuildMarker> {
        self.guild
    }

    pub fn prefix(&self) -> &String {
        &self.prefix
    }

    pub fn redis(&self) -> &String {
        &self.redis
    }

    pub fn role(&self) -> Id<RoleMarker> {
        self.role
    }

    pub fn token(&self) -> String {
        self.token.clone()
    }
}
