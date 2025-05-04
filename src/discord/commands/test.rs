use std::sync::Arc;

use twilight_model::channel::Message;

use crate::{
    di::DI,
    discord::{DiscordHelper, ElectionHelper},
};

pub async fn text(args: Arc<(Message, Vec<String>, DI)>) -> Result<(), crate::Error> {
    let (message, _, di) = args.as_ref();
    let guild = DiscordHelper::guild_from_id(di.clone(), message.guild_id.clone().unwrap())
        .await
        .unwrap();
    let member = DiscordHelper::guild_member_from_ids(
        di.clone(),
        message.guild_id.clone().unwrap(),
        message.author.id.clone(),
    )
    .await
    .unwrap();

    let _ =
        ElectionHelper::dm_user_with_nomination(di.clone(), guild.clone(), member.clone()).await;
    Ok(())
}
