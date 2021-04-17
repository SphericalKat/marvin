#![feature(destructuring_assignment)]
use dotenv::dotenv;
use handlers::{banning, misc, save_chat_handler, save_user_handler};
use lazy_static::lazy_static;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::sync::Arc;
use teloxide::{
    adaptors::DefaultParseMode, prelude::*, types::ParseMode, utils::command::BotCommand,
};

pub mod entities;
pub mod handlers;
pub mod repo;
pub mod utils;

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "List of supported commands:")]
enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "Get a user's ID.")]
    Id,
    #[command(description = "Ban a user.")]
    Ban,
    #[command(description = "Temporarily ban a user.")]
    Tban,
    #[command(description = "Kick a user")]
    Kick,
}

type Cx = UpdateWithCx<Arc<DefaultParseMode<AutoSend<Bot>>>, Message>;

lazy_static! {
    static ref DATABASE_URL: String = std::env::var("DATABASE_URL").expect("Expected database url");
    static ref POOL: Pool<Postgres> = PgPoolOptions::new()
        .max_connections(10)
        .connect_lazy(&DATABASE_URL)
        .unwrap();
    static ref BOT_ID: i64 = std::env::var("BOT_ID")
        .expect("Expected bot token")
        .parse::<i64>()
        .expect("Expected bot token to be numeric");
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run().await
}

async fn handler(cx: Cx) -> anyhow::Result<()> {
    // opportunistically save user/chat details to db
    tokio::try_join!(
        save_user_handler(&cx, &*POOL),
        save_chat_handler(&cx, &*POOL)
    )?;

    // check if update contains any text
    let text = cx.update.text();
    if text.is_none() {
        return Ok(());
    }

    let cmd = Command::parse(text.unwrap(), "rust_tgbot");

    match cmd {
        Ok(c) => match c {
            Command::Help => cx
                .reply_to(Command::descriptions())
                .send()
                .await
                .map(|_| ())?,
            Command::Id => {
                misc::handle_id(cx, &*POOL).await?;
            }
            Command::Ban => {
                banning::ban(cx, false, &*POOL).await?;
            }
            Command::Tban => {
                banning::ban(cx, true, &*POOL).await?;
            }
            Command::Kick => {
                banning::kick(cx, &*POOL).await?;
            }
        },
        Err(_) => {}
    }

    Ok(())
}

async fn run() -> anyhow::Result<()> {
    // load env config
    dotenv()?;
    teloxide::enable_logging!();

    // start bot
    log::info!("Starting marvin...");
    let bot = Arc::new(Bot::from_env().auto_send().parse_mode(ParseMode::Html));

    teloxide::repl(bot.clone(), handler).await;

    Ok(())
}
