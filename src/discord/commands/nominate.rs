use std::sync::Arc;

use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use twilight_model::channel::Message;

use crate::{
    db,
    di::DI,
    discord::{DiscordHelper, ElectionHelper},
};

pub async fn text(args: Arc<(Message, Vec<String>, DI)>) -> Result<(), crate::Error> {
    let (message, args, di) = args.as_ref().clone();

    if args.len() == 0 {
        DiscordHelper::reply(di.clone(), message, "Please enter a user to nominate!");
        return Ok(());
    }

    if let Some(user) =
        DiscordHelper::find_first_user_in(di.clone(), message.guild_id.unwrap(), args.clone()).await
    {
        // find current election, TODO: this should be cached sometime

        let mut conn = di.db.get()?;
        let current_election = {
            use db::models::Election;
            use db::schema::elections::dsl::*;

            match elections
                .filter(status.eq("scheduled"))
                .filter(server_id.eq(message.guild_id.unwrap().to_string()))
                .first::<Election>(&mut conn)
            {
                Ok(election) => election,
                Err(_) => {
                    DiscordHelper::reply(
                        di.clone(),
                        message.clone(),
                        &format!("There is no scheduled election for this server!"),
                    )
                    .await;
                    return Ok(());
                }
            }
        };
        let possible_nominee = {
            use db::models::{NewNominee, Nominee};
            use db::schema::nominees::dsl::*;

            nominees
                .filter(election_id.eq(current_election.uuid.clone()))
                .filter(user_id.eq(user.user.id.to_string()))
                .first::<Nominee>(&mut conn)
        };
        match possible_nominee {
            Ok(nominee) => {}
            Err(_) => {
                use db::models::{NewNominee, Nominee};
                use db::schema::nominees::dsl::*;

                // time to nominate the user!
                let new_nominee = NewNominee {
                    election_id: current_election.uuid.clone(),
                    user_id: user.user.id.to_string(),
                    poll_option_text: DiscordHelper::guild_member_name(&user),
                    votes_received: None,
                    nomination_status: String::from("pending"),
                };
                let inserted_nominee = diesel::insert_into(nominees)
                    .values(&new_nominee)
                    .get_result::<Nominee>(&mut conn)
                    .expect("Failed to insert new nominee");
                let guild = di
                    .discord_http
                    .guild(message.guild_id.unwrap())
                    .await
                    .expect("message.guild_id should resolve to a guild")
                    .model()
                    .await
                    .expect("discord should return a proper guild model");
                ElectionHelper::dm_user_with_nomination(di.clone(), guild.clone(), user.clone())
                    .await;
            }
        }
        DiscordHelper::reply(
            di.clone(),
            message,
            &format!(
                "{} has been nominated! (TODO)",
                &user
                    .nick
                    .or(user.user.global_name)
                    .unwrap_or(user.user.name)
            ),
        )
        .await;
    }

    Ok(())
}
