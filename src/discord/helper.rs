use regex::Regex;
use string_patterns::PatternCapture;
use twilight_model::{
    channel::Message,
    guild::{Guild, Member},
    id::{
        marker::{GuildMarker, UserMarker},
        Id,
    },
};

use crate::di::DI;

pub struct DiscordHelper;

impl DiscordHelper {
    pub async fn reply(di: DI, to: Message, content: &str) {
        let _ = di
            .discord_http
            .create_message(to.channel_id)
            .reply(to.id)
            .content(content)
            .await;
    }
    pub async fn find_first_user_in(
        di: DI,
        server_id: Id<GuildMarker>,
        arg_list: Vec<String>,
    ) -> Option<Member> {
        let reg = Regex::new(r"<@([0-9]+)>").unwrap();
        for arg in arg_list {
            if let Some(caps) = reg.captures(&arg)
                && let Some(id) = caps.get(1)
            {
                // log::trace!("looking for id {}", id.as_str());
                let id = id.as_str().parse().unwrap(); // we already know it's a valid number per regex
                if let Ok(user) = di.discord_http.guild_member(server_id, Id::new(id)).await
                    && let Ok(member) = user.model().await
                {
                    log::trace!("found member {}", member.user.id);
                    return Some(member);
                }
            }
        }
        return None;
    }
    pub fn guild_member_name(user: &Member) -> String {
        let user = user.clone();
        return user
            .nick
            .or(user.user.global_name)
            .unwrap_or(user.user.name);
    }
}

pub struct ElectionHelper;

impl ElectionHelper {
    pub async fn dm_user_with_nomination(di: DI, guild: Guild, user: Member) {
        // i love discord components v2!!
    }
}
