use std::sync::Arc;

use anyhow::Context;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use twilight_interactions::command::{self, CommandModel, CreateCommand, ResolvedUser};
use twilight_model::{
    application::interaction::{application_command::CommandData, Interaction},
    channel::Message,
    id::{
        marker::{GuildMarker, UserMarker},
        Id,
    },
};

use crate::{
    db,
    di::DI,
    discord::{DiscordHelper, ElectionHelper, InteractionHelper},
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
        let di = di.clone();
        let msg = message.clone();
        handle_nominate_command(
            di.clone(),
            message.clone().guild_id.unwrap(),
            message.clone().author.id,
            async move |s| {
                DiscordHelper::reply(di.clone(), msg.clone(), &s).await;
                Ok(())
            },
        )
        .await;
    }

    Ok(())
}
#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "nominate", desc = "Nominate a user for an election")]
pub struct NominateCommand {
    /// The user to nominate
    user: Option<ResolvedUser>,
}
impl NominateCommand {
    pub async fn handle(
        di: DI,
        interaction: Interaction,
        data: CommandData,
    ) -> Result<(), crate::Error> {
        if interaction.guild_id.is_none() {
            InteractionHelper::update_response(
                di.clone(),
                interaction.clone(),
                "You can only run this command from inside a server!".to_string(),
                vec![],
            )
            .await;
            return Ok(());
        }
        let command = NominateCommand::from_interaction(data.clone().into())
            .context("failed to parse command data")?;
        let user = command
            .user
            .map(|x| x.resolved.id)
            .unwrap_or(interaction.member.clone().unwrap().user.unwrap().id);
        handle_nominate_command(
            di.clone(),
            interaction.guild_id.clone().unwrap(),
            user,
            async move |s| {
                InteractionHelper::update_response(di.clone(), interaction.clone(), s, vec![])
                    .await;
                Ok(())
            },
        )
        .await;
        Ok(())
    }
}

async fn handle_nominate_command<Func>(
    di: DI,
    guild_id: Id<GuildMarker>,
    the_user_id: Id<UserMarker>,
    reply_fn: Func,
) where
    Func: async Fn(String) -> Result<(), crate::Error>,
{
    let mut conn = di.db_pool.get().unwrap();
    let mut user = di
        .discord_http
        .guild_member(guild_id, the_user_id)
        .await
        .unwrap()
        .model()
        .await
        .unwrap();
    let current_election = {
        use db::models::Election;
        use db::schema::elections::dsl::*;

        match elections
            .filter(status.eq("scheduled"))
            .filter(server_id.eq(guild_id.to_string()))
            .first::<Election>(&mut conn)
        {
            Ok(election) => election,
            Err(_) => {
                reply_fn(format!("There is no scheduled election for this server!")).await;
                return;
            }
        }
    };
    let possible_nominee = {
        use db::models::{NewNominee, Nominee};
        use db::schema::nominees::dsl::*;

        nominees
            .filter(election_id.eq(current_election.uuid.clone()))
            .filter(user_id.eq(the_user_id.to_string()))
            .first::<Nominee>(&mut conn)
    };
    match possible_nominee {
        Ok(nominee) => match nominee.nomination_status.as_str() {
            "pending" => {
                reply_fn(format!(
                    "{} has already been invited to accept their nomination!",
                    DiscordHelper::guild_member_name(&user)
                ))
                .await;
            }
            "accepted" => {
                reply_fn(format!(
                    "{} has already been accepted!",
                    DiscordHelper::guild_member_name(&user)
                ))
                .await;
            }
            "denied" => {
                reply_fn(format!(
                    "{} has already denied their nomination!",
                    DiscordHelper::guild_member_name(&user)
                ))
                .await;
            }
            _ => {
                unreachable!();
            }
        },
        Err(_) => {
            use db::models::{NewNominee, Nominee};
            use db::schema::nominees::dsl::*;

            // time to nominate the user!
            let new_nominee = NewNominee {
                election_id: current_election.uuid.clone(),
                user_id: the_user_id.to_string(),
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
                .guild(guild_id)
                .await
                .expect("guild_id should resolve to a guild")
                .model()
                .await
                .expect("discord should return a proper guild model");
            ElectionHelper::dm_user_with_nomination(di.clone(), guild.clone(), user.clone()).await;
            reply_fn(format!(
                "{} has been nominated!",
                DiscordHelper::guild_member_name(&user)
            ))
            .await;
        }
    }
}
