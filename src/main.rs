use dotenv::dotenv;
use handlers::{admin, banning, filters, misc, muting, save_chat_handler, save_user_handler};
use lazy_static::lazy_static;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use teloxide::{
    adaptors::DefaultParseMode,
    dispatching2::UpdateFilterExt,
    prelude2::*,
    types::{ChatAction, ParseMode},
    utils::command::BotCommand,
};
use utils::PinMode;

pub mod entities;
pub mod handlers;
pub mod repo;
pub mod utils;

type Bot = AutoSend<DefaultParseMode<teloxide::Bot>>;

#[derive(BotCommand, Clone)]
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
    #[command(
        description = "Get a saved note in this chat. Can be used as <code>#notename</code> or <code>/get notename</code>"
    )]
    Get,
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

async fn save_details(bot: &Bot, message: &Message) -> anyhow::Result<()> {
    // opportunistically save user/chat details to db
    tokio::try_join!(
        save_user_handler(bot, message, &*POOL),
        save_chat_handler(bot, message, &*POOL)
    )?;

    Ok(())
}

async fn answer(bot: Bot, message: Message) -> anyhow::Result<()> {
    save_details(&bot, &message).await?;

    // check if update contains any text
    let text = message.text();
    if text.is_none() {
        return Ok(());
    }

    // check if string begins with
    if text.unwrap().starts_with('#') && text.unwrap().split_whitespace().count() < 2 {
        filters::get_note(&bot, &message, false, &*POOL).await?;
    }

    let cmd = Command::parse(text.unwrap(), "rust_tgbot").ok();

    if cmd.is_some() {
        match cmd.unwrap() {
            Command::Help => {
                bot.send_chat_action(message.chat.id, ChatAction::Typing)
                    .await?;
                bot.send_message(message.chat.id, Command::descriptions())
                    .await?;
            }
            Command::Id => {
                misc::handle_id(&bot, &message, &*POOL).await?;
            }
            Command::Ban => {
                banning::ban(&bot, &message, false, &*POOL).await?;
            }
            Command::Tban => {
                banning::ban(&bot, &message, true, &*POOL).await?;
            }
            Command::Kick => {
                banning::kick(&bot, &message, &*POOL).await?;
            }
            Command::Kickme => {
                banning::kickme(&bot, &message).await?;
            }
            Command::Unban => {
                banning::unban(&bot, &message, &*POOL).await?;
            }
            Command::Mute => {
                muting::mute(&bot, &message, false, &*POOL).await?;
            }
            Command::Tmute => {
                muting::mute(&bot, &message, true, &*POOL).await?;
            }
            Command::Unmute => {
                muting::unmute(&bot, &message, &*POOL).await?;
            }
            Command::Promote => {
                admin::promote(&bot, &message, &*POOL).await?;
            }
            Command::Demote => {
                admin::demote(&bot, &message, &*POOL).await?;
            }
            Command::Pin(mode) => {
                admin::pin(&bot, &message, mode).await?;
            }
            Command::Invitelink => {
                admin::invite(&bot, &message).await?;
            }
            Command::Save => {
                filters::save_note(&bot, &message, &*POOL).await?;
            }
            Command::Get => {
                filters::get_note(&bot, &message, true, &*POOL).await?;
            }
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
    let bot = teloxide::Bot::from_env()
        .parse_mode(ParseMode::Html)
        .auto_send();

    let handler = dptree::entry().branch(Update::filter_message().endpoint(answer));

    Dispatcher::builder(bot, handler)
        .build()
        .setup_ctrlc_handler()
        .dispatch()
        .await;

    log::info!("Shutting down marvin... goodbye!");

    Ok(())
}
