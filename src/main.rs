use dotenv::dotenv;
use teloxide::{prelude::*, utils::command::BotCommand};
use refinery::config::Config;

pub mod migrations;

#[derive(BotCommand)]
#[command(rename = "lowercase", description = "List of supported commands:")]
enum Command {
    #[command(description = "display this text.")]
    Help,
}

type Cx = UpdateWithCx<AutoSend<Bot>, Message>;

#[tokio::main]
async fn main() {
    run().await;
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
            // non-command update, perform data collection here
        }
    }

    Ok(())
}

async fn run() {
    // load env config
    dotenv().ok();
    teloxide::enable_logging!();

    // run migrations
    log::info!("Running database migrations...");
    let mut db_conf = Config::from_env_var("DATABASE_URL").expect("DATABASE_URL is required");
    
    let bs = migrations::runner().run_async(&mut db_conf).await.unwrap();
    log::debug!("Ran migrations: {:?}", bs.applied_migrations());

    log::info!("Starting marvin...");

    let bot = Bot::from_env().auto_send();
    teloxide::repl(bot, handler).await;
}
