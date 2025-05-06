use chrono::{DateTime, Utc};
use diesel::prelude::*;

use super::schema::*;

#[derive(
    Queryable, Selectable, Insertable, Associations, Identifiable, Debug, Clone, AsChangeset,
)]
#[diesel(table_name = elections)]
#[diesel(belongs_to(Server, foreign_key = server_id))]
#[diesel(primary_key(uuid))]
pub struct Election {
    pub uuid: String,
    pub server_id: String,
    pub poll_message_id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,

    pub status: String,
}

#[derive(
    Queryable, Selectable, Insertable, Associations, Identifiable, Debug, Clone, AsChangeset,
)]
#[diesel(table_name = nominees)]
#[diesel(belongs_to(Election, foreign_key = election_id))]
#[diesel(primary_key(id))]
pub struct Nominee {
    #[diesel(deserialize_as = i32)]
    pub id: i32,
    pub election_id: String,
    pub user_id: String,
    pub poll_option_text: String,
    pub votes_received: Option<i32>,
    pub nomination_status: String,
}

#[derive(Queryable, Selectable, Insertable, Identifiable, Debug, Clone, AsChangeset)]
#[diesel(table_name = servers)]
#[diesel(primary_key(id))]
pub struct Server {
    pub id: String,
    pub announcements_channel_id: Option<String>,
    pub poll_channel_id: Option<String>,
    pub election_frequency: i32,
    pub active: bool,
    pub winner_temp_role_id: Option<String>,
    pub winner_perm_role_id: Option<String>,
}

#[derive(Insertable, Debug, Clone)]
#[diesel(table_name = nominees)]
pub struct NewNominee {
    pub election_id: String,
    pub user_id: String,
    pub poll_option_text: String,
    pub votes_received: Option<i32>,
    pub nomination_status: String,
}

impl Server {
    pub fn default_with_id(server_id: String) -> Self {
        Server {
            id: server_id,
            announcements_channel_id: None,
            poll_channel_id: None,
            election_frequency: 336,
            active: false,
            winner_temp_role_id: None,
            winner_perm_role_id: None,
        }
    }
}
