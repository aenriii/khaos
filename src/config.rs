use serde::Deserialize;
use twilight_model::id::{
    marker::{GuildMarker, RoleMarker},
    Id,
};

// I did a funny. ~ahill
#[derive(Clone, Deserialize)]
pub(crate) struct KhaosControl {
    duration: u64,
    epoch: u64,
    guild: Id<GuildMarker>,
    interval: u64,
    prefix: String,
    redis: String,
    role: Id<RoleMarker>,
    token: String,
}

impl KhaosControl {
    pub fn duration(&self) -> u64 {
        self.duration
    }

    pub fn epoch(&self) -> u64 {
        self.epoch.clone()
    }

    pub fn guild(&self) -> Id<GuildMarker> {
        self.guild
    }

    pub fn interval(&self) -> u64 {
        self.interval
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
