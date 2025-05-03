use std::sync::Arc;

use twilight_model::channel::Message;

use crate::{di::DI, discord::DiscordHelper};

pub async fn text(args: Arc<(Message, Vec<String>, DI)>) -> Result<(), crate::Error> {
    let (message, _, di) = args.as_ref();
    DiscordHelper::reply(di.clone(), message.clone(), "Pong!").await;
    Ok(())
}
