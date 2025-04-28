mod db;

use std::sync::Arc;

use dotenvy::{dotenv, from_path};
#[tokio::main]
async fn main() {
    #[cfg(debug_assertions)]
    {
        println!("Loading dev.env!");
        dotenv().ok();
        from_path("dev.env").ok();
    }
    #[cfg(not(debug_assertions))]
    {
        dotenv().ok();
    }

    let _discord_token = dotenvy::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be set");

    let _connection = match db::establish_pool() {
        Ok(pool) => Arc::new(pool),
        Err(err) => {
            eprintln!("Failed to establish database connection: {}", err);
            std::process::exit(1);
        }
    };

    // finally, the bot.

    // TODO: that
}
