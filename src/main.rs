use dotenv::dotenv;
use handlers::{admin, banning, filters, misc, muting, save_chat_handler, save_user_handler};
use lazy_static::lazy_static;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use teloxide::{
    prelude2::*,
    types::{ChatAction, ParseMode},
    utils::command::BotCommand,
};
use utils::PinMode;

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
    #[command(description = "Kick yourself")]
    Kickme,
    #[command(description = "Unban a user")]
    Unban,
    #[command(description = "Mute a user.")]
    Mute,
    #[command(description = "Temporarily mute a user.")]
    Tmute,
    #[command(description = "Unmute a user.")]
    Unmute,
    #[command(description = "Promote a user")]
    Promote,
    #[command(description = "Demote a user")]
    Demote,
    #[command(description = "Pin a message")]
    Pin(PinMode),
    #[command(description = "Get the chat's invite link")]
    Invitelink,
    #[command(description = "Save a note in this chat")]
    Save,
}

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

async fn handler(bot: AutoSend<Bot>, message: Message, command: Command) -> anyhow::Result<()> {
    // opportunistically save user/chat details to db
    tokio::try_join!(
        save_user_handler(&bot, &message, &*POOL),
        save_chat_handler(&bot, &message, &*POOL)
    )?;

    // check if update contains any text
    let text = message.text();
    if message.text().is_none() {
        return Ok(());
    }

    match command {
        Command::Help => {
            bot.send_chat_action(message.chat.id, ChatAction::Typing)
                .await?;
            bot.send_message(message.chat.id, Command::descriptions())
                .await?;
        }
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
        Command::Kickme => {
            banning::kickme(cx).await?;
        }
        Command::Unban => {
            banning::unban(cx, &*POOL).await?;
        }
        Command::Mute => {
            muting::mute(cx, false, &*POOL).await?;
        }
        Command::Tmute => {
            muting::mute(cx, true, &*POOL).await?;
        }
        Command::Unmute => {
            muting::unmute(cx, &*POOL).await?;
        }
        Command::Promote => {
            admin::promote(cx, &*POOL).await?;
        }
        Command::Demote => {
            admin::demote(cx, &*POOL).await?;
        }
        Command::Pin(mode) => {
            admin::pin(cx, mode).await?;
        }
        Command::Invitelink => {
            admin::invite(cx).await?;
        }
        Command::Save => {
            filters::save_note(cx, &*POOL).await?;
        }
    }

    Ok(())
}

async fn run() -> anyhow::Result<()> {
    // load env config
    dotenv()?;
    teloxide::enable_logging!();

    // start bot
    log::info!("Starting marvin...");
    let bot = Bot::from_env().auto_send();

    teloxide::repls2::commands_repl(bot, handler, Command::ty()).await;

    Ok(())
}
