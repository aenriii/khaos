use regex::Regex;
use string_patterns::PatternCapture;
use twilight_http::request::channel::message::CreateMessage;
use twilight_model::{
    channel::{message::MessageFlags, Message},
    gateway::payload::incoming::InteractionCreate,
    guild::{Guild, Member},
    http::{
        attachment::Attachment,
        interaction::{InteractionResponse, InteractionResponseData, InteractionResponseType},
    },
    id::{
        marker::{GuildMarker, InteractionMarker, UserMarker},
        Id,
    },
};
use twilight_util::builder::InteractionResponseDataBuilder;

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
    pub async fn guild_from_id(di: DI, id: Id<GuildMarker>) -> Option<Guild> {
        let guild = di.discord_http.guild(id).await;
        if let Ok(guild) = guild {
            let model = guild.model().await;
            if let Ok(model) = model {
                log::trace!("found guild {}", model.id);
                return Some(model);
            }
        }
        None
    }
    pub async fn guild_member_from_ids(
        di: DI,
        guild_id: Id<GuildMarker>,
        user_id: Id<UserMarker>,
    ) -> Option<Member> {
        let member = di.discord_http.guild_member(guild_id, user_id).await;
        if let Ok(member) = member {
            let model = member.model().await;
            if let Ok(model) = model {
                log::trace!("found member {}", model.user.id);
                return Some(model);
            }
        }
        None
    }
}

pub struct ElectionHelper;

impl ElectionHelper {
    pub async fn dm_user_with_nomination(di: DI, guild: Guild, user: Member) {
        // i love discord components v2!!
        log::trace!("dm_user_with_nomination");
        let dm_channels = di
            .discord_http
            .create_private_channel(user.user.id)
            .await
            .unwrap()
            .model()
            .await
            .unwrap();
        log::trace!("Created DM channel with user {}", user.user.id);
        let payload = format!(
            r#"
            {{
                "flags": 32768,
                "components": [
                    {{
                        "type": 17,
                        "components": [
                            {{
                                "type": 10,
                                "content": " # Hello, {}!"
                            }},
                            {{
                                "type": 10,
                                "content": "You've been nominated in {}!"
                            }},
                            {{
                                "type": 14,
                                "spacing": 2
                            }},
                            {{
                                "type": 1,
                                "components": [
                                    {{
                                        "type": 2,
                                        "label": "Accept Nomination",
                                        "style": 1,
                                        "custom_id": "accept_nomination_uid{}_gid{}"
                                    }},
                                    {{
                                        "type": 2,
                                        "label": "Decline Nomination",
                                        "style": 4,
                                        "custom_id": "decline_nomination_uid{}_gid{}"
                                    }}
                                ]
                            }}
                        ]
                    }}
                ]
            }}
            "#,
            DiscordHelper::guild_member_name(&user),
            &guild.name,
            &user.user.id,
            &guild.id,
            &user.user.id,
            &guild.id
        );
        println!("{}", payload);
        let message = di
            .discord_http
            .create_message(dm_channels.id)
            .payload_json(&payload.into_bytes().into_boxed_slice())
            .await;
        match message {
            Ok(message) => log::trace!("dm_user_with_nomination: success!"),
            Err(err) => log::error!("dm_user_with_nomination: {}", err),
        }
    }
}

pub struct InteractionHelper;

impl InteractionHelper {
    pub async fn set_callback_deferred(di: DI, id: Id<InteractionMarker>, token: String) {
        let _ = di
            .discord_http
            .interaction(di.current_application_id.get().unwrap().clone())
            .create_response(
                id,
                &token,
                &InteractionResponse {
                    kind: InteractionResponseType::DeferredChannelMessageWithSource,
                    data: None,
                },
            )
            .await;
    }
    pub async fn update_response(
        di: DI,
        interaction: InteractionCreate,
        content: String,
        attachments: Vec<Attachment>,
    ) {
        let _ = di
            .discord_http
            .interaction(di.current_application_id.get().unwrap().clone())
            .update_response(&interaction.token)
            .content(Some(&content))
            .attachments(&attachments)
            .await;
    }
}
