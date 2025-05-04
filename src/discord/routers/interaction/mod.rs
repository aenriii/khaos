use std::sync::Arc;

use handler::{InteractionFilterFn, InteractionHandler, InteractionHandlerFn};
use twilight_model::{gateway::payload::incoming::InteractionCreate, http::attachment::Attachment};
use twilight_util::builder::{command::AttachmentBuilder, InteractionResponseDataBuilder};

use crate::{di::DI, discord::InteractionHelper};

mod handler;

#[derive(Debug)]
pub struct InteractionRouter {
    pub handlers: Vec<InteractionHandler>,
    di: DI,
}

impl InteractionRouter {
    pub fn new(di: DI) -> Self {
        Self {
            handlers: Vec::new(),
            di,
        }
    }
    pub fn add_handler(&mut self, filter: InteractionFilterFn, handler: InteractionHandlerFn) {
        self.handlers.push(InteractionHandler::new(filter, handler));
    }
    pub async fn route(&self, interaction: InteractionCreate) {
        let args = Arc::new((self.di.clone(), interaction.clone()));
        InteractionHelper::set_callback_deferred(
            self.di.clone(),
            interaction.id.clone(),
            interaction.token.clone(),
        )
        .await;
        for handler in &self.handlers {
            if handler.filter(args.clone()).await {
                match handler.handle(args).await {
                    Ok(_) => (),
                    Err(err) => {
                        log::error!("Failed to handle interaction: {}", err);
                        let _ = InteractionHelper::update_response(
                            self.di.clone(),
                            interaction.clone().0,
                            String::from("Oops! We had an error handling that request for some reason, give the following file to a bot developer!"),
                            vec![
                                Attachment::from_bytes(String::from("failed_interaction.rs-log"), format!(r#"Failed to handle interaction with data:
{:?}

Error: {}
"#, interaction.clone(), err).into(), 1)
                            ]        ).await;
                    }
                }
                return;
            }
        }
        // if no handler was found, something went wrong
        let _ = InteractionHelper::update_response(
            self.di.clone(),
            interaction.clone().0,
            String::from("Oops! We couldn't handle that request for some reason, give the following file to a bot developer!"),
            vec![
                Attachment::from_bytes(String::from("failed_interaction.rs-log"), format!(r#"Failed to handle interaction with data:
{:?}"#, interaction.clone()).into(), 1)
            ]        ).await;
    }
}
