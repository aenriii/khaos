use anyhow::Context;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use twilight_interactions::command::{self, CommandModel, CreateCommand, ResolvedUser};
use twilight_model::{
    application::interaction::{application_command::CommandData, Interaction, InteractionChannel},
    channel::Message,
    guild::Role,
    id::{
        marker::{GuildMarker, UserMarker},
        Id,
    },
};

use crate::{
    db::{self, schema::servers},
    di::DI,
    discord::InteractionHelper,
};

#[derive(Debug, Clone, CommandModel, CreateCommand)]
#[command(name = "setup", desc = "Setup your server for elections!")]
pub struct SetupCommand {
    /// Channel to put announcements in, such as
    /// who won or when nominations are open
    announcements_channel: InteractionChannel,
    /// The role to give the person who wins the
    /// election temporarily while they're the
    /// president. The bot must be above this
    /// role
    winner_role: Role,
    /// Channel to put polls in, this defaults
    /// to the same channel as announcements
    poll_channel: Option<InteractionChannel>,
    /// The frequency, in hours, of elections
    /// defaults to 336, or two weeks
    election_frequency: Option<i64>,
    /// The role to give people who have won
    /// the election in the past but are not the
    /// current winner
    winner_permanent_role: Option<Role>,
}

impl SetupCommand {
    pub async fn handle(
        di: DI,
        interaction: Interaction,
        data: CommandData,
    ) -> Result<(), crate::Error> {
        if interaction.guild_id.is_none() {
            InteractionHelper::update_response(
                di.clone(),
                interaction,
                "You can only run this command from inside a server!".to_string(),
                vec![],
            )
            .await;
            return Ok(());
        }
        let guild_id = interaction
            .guild_id
            .expect("already verified guild id is not none");
        let command = SetupCommand::from_interaction(data.clone().into())
            .context("failed to parse command data")?;

        let mut db = match di.db_pool.get() {
            Ok(it) => it,
            Err(e) => return Err(format!("Failed to get DB connection, {}", e).into()),
        };

        let mut action = "update";

        let mut server = {
            use db::models::Server;
            use db::schema::servers::dsl::*;

            match servers
                .filter(id.eq(guild_id.to_string()))
                .first::<Server>(&mut db)
            {
                Ok(server) => server,
                Err(_) => {
                    action = "insert";
                    Server::default_with_id(guild_id.to_string())
                }
            }
        };

        server.announcements_channel_id = Some(command.announcements_channel.id.to_string());
        server.poll_channel_id = Some(
            command
                .poll_channel
                .map_or(command.announcements_channel.id.to_string(), |chn| {
                    chn.id.to_string()
                }),
        );
        server.election_frequency = command
            .election_frequency
            .unwrap_or(server.election_frequency as i64) as i32;

        server.winner_temp_role_id = Some(command.winner_role.id.to_string());
        server.winner_perm_role_id = command.winner_permanent_role.map(|rl| rl.id.to_string());

        if action == "update" {
            diesel::update(&server).set(server.clone()).execute(&mut db);
        } else if action == "insert" {
            diesel::insert_into(servers::table)
                .values(server)
                .execute(&mut db);
        } else {
            return Err(format!("unreachable! action == {}", action).into());
        }

        InteractionHelper::update_response(
            di,
            interaction,
            format!("Updated server settings!"),
            vec![],
        )
        .await;
        Ok(())
    }
}
