use std::{
    ops::Deref,
    sync::{Arc, LazyLock},
};

use diesel::{update, ExpressionMethods, QueryDsl, RunQueryDsl};
use twilight_model::{
    application::interaction::{InteractionData, InteractionType},
    gateway::payload::incoming::InteractionCreate,
};

use crate::{
    db::{self, models::NewNominee},
    di::DI,
    discord::InteractionHelper,
    Error,
};
static REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r#"(accept|decline)_nomination_uid([0-9]+)_gid([0-9]+)"#).unwrap()
});

pub async fn filter(args: Arc<(DI, InteractionCreate)>) -> Result<bool, Error> {
    let (_, interaction) = args.deref();
    let interaction = interaction.clone();
    match interaction.data.clone() {
        Some(InteractionData::MessageComponent(data)) => {
            // log::trace!("Received message component interaction");
            // log::trace!("Custom ID: {}", data.custom_id);

            return Ok(REGEX.is_match(&data.custom_id));
        }
        _ => return Ok(false),
    }
}

pub async fn handle(args: Arc<(DI, InteractionCreate)>) -> Result<(), Error> {
    let (di, interaction) = args.deref().clone();
    let interaction = interaction.clone();
    match interaction.data.clone() {
        Some(InteractionData::MessageComponent(data)) => {
            let captures = REGEX.captures(&data.custom_id).unwrap();
            let action = captures.get(1).unwrap().as_str();
            let uid = captures.get(2).unwrap().as_str().parse::<u64>().unwrap();
            let gid = captures.get(3).unwrap().as_str().parse::<u64>().unwrap();

            let mut db = match di.db.get() {
                Ok(db) => db,
                Err(err) => {
                    log::error!("Failed to get database connection: {}", err);
                    return Err(String::from("Failed to get database connection").into());
                }
            };
            if action != "accept" && action != "decline" {
                log::error!("Invalid action: {}", action);
                return Err(String::from("Invalid action").into());
            }

            let current_election = {
                use db::models::Election;
                use db::schema::elections::dsl::*;
                match elections
                    .filter(status.eq("scheduled"))
                    .filter(server_id.eq(gid.to_string()))
                    .first::<Election>(&mut db)
                {
                    Ok(election) => election,
                    Err(err) => {
                        InteractionHelper::update_response(
                            di.clone(),
                            interaction.clone().0,
                            format!(
                                "Failed to {} the nomination, as there is no scheduled election!",
                                action
                            ),
                            vec![],
                        )
                        .await;
                        return Ok(());
                    }
                }
            };

            let mut nominated_user = {
                use db::models::Nominee;
                use db::schema::nominees::dsl::*;
                nominees
                    .filter(election_id.eq(current_election.uuid.clone()))
                    .filter(user_id.eq(uid.to_string()))
                    .first::<Nominee>(&mut db)
            };
            use db::models::Nominee;
            use db::schema::nominees::dsl::*;

            if action == "accept" {
                match nominated_user {
                    Ok(mut nominee) => {
                        update(&nominee)
                            .set(nomination_status.eq("accepted"))
                            .execute(&mut db);
                        InteractionHelper::update_response(
                            di.clone(),
                            interaction.clone().0,
                            String::from("Nomination accepted"),
                            vec![],
                        )
                        .await;
                    }
                    Err(err) => {
                        InteractionHelper::update_response(
                            di.clone(),
                            interaction.clone().0,
                            String::from("Nomination not found"),
                            vec![],
                        )
                        .await;
                        return Ok(());
                    }
                }
            } else if action == "decline" {
                match nominated_user {
                    Ok(mut nominee) => {
                        update(&nominee)
                            .set(nomination_status.eq("declined"))
                            .execute(&mut db);
                        InteractionHelper::update_response(
                            di.clone(),
                            interaction.clone().0,
                            String::from("Nomination declined"),
                            vec![],
                        )
                        .await;
                    }
                    Err(err) => {
                        InteractionHelper::update_response(
                            di.clone(),
                            interaction.clone().0,
                            String::from("Failed to decline the nomination"),
                            vec![],
                        )
                        .await;
                        return Ok(());
                    }
                }
            }
        }
        _ => return Ok(()),
    };
    Err(String::from("Invalid interaction type").into())
}
