use serde::Deserialize;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
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
    pub fn duration(&self) -> Duration {
        Duration::from_secs(self.duration)
    }

    pub fn epoch(&self) -> SystemTime {
        UNIX_EPOCH.checked_add(Duration::from_secs(self.epoch)).unwrap()
    }

    pub fn guild(&self) -> Id<GuildMarker> {
        self.guild
    }

    pub fn interval(&self) -> Duration {
        Duration::from_secs(self.interval)
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
