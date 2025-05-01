use twilight_model::channel::Message;

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
}
