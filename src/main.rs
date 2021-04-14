use dotenv::dotenv;
use handlers::{save_chat_handler, save_user_handler};
use lazy_static::lazy_static;
use refinery::config::Config;
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use teloxide::{prelude::*, utils::command::BotCommand};

pub mod entities;
pub mod handlers;
pub mod migrations;
pub mod repo;

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "List of supported commands:")]
enum Command {
    #[command(description = "display this text.")]
    Help,
}

type Cx = UpdateWithCx<AutoSend<Bot>, Message>;

lazy_static! {
    static ref DATABASE_URL: String = std::env::var("DATABASE_URL").expect("Expected database url");
    static ref POOL: Pool<Postgres> = PgPoolOptions::new()
        .max_connections(10)
        .connect_lazy(&DATABASE_URL)
        .unwrap();
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    run().await
}

async fn handler(cx: Cx) -> anyhow::Result<()> {
    log::debug!("Received an update!");

    // check if update contains any text
    let text = cx.update.text();
    if text.is_none() {
        return Ok(());
    }

    let cmd = Command::parse(text.unwrap(), "marvin");

    match cmd {
        Ok(c) => match c {
            Command::Help => cx
                .answer(Command::descriptions())
                .send()
                .await
                .map(|_| ())?,
        },
        Err(_) => {
            tokio::try_join!(
                save_user_handler(&cx, &*POOL),
                save_chat_handler(&cx, &*POOL)
            )?;
        }
    }

    Ok(())
}

async fn run() -> anyhow::Result<()> {
    // load env config
    dotenv()?;
    teloxide::enable_logging!();

    // run migrations
    log::info!("Running database migrations...");
    let mut db_conf = Config::from_env_var("DATABASE_URL")?;

    let bs = migrations::runner().run_async(&mut db_conf).await?;
    log::debug!("Ran migrations: {:?}", bs.applied_migrations());

    // start bot
    log::info!("Starting marvin...");
    let bot = Bot::from_env().auto_send();
    teloxide::repl(bot, handler).await;

    Ok(())
}
