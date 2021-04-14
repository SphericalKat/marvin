use dotenv::dotenv;
use teloxide::{prelude::*, utils::command::BotCommand};
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
        }
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
    log::info!("Starting marvin...");

    let bot = Bot::from_env().auto_send();
    teloxide::repl(bot, handler).await;
}
