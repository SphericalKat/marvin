use crate::{Cx, utils};
use crate::utils::UnitOfTime;
use tokio_cron_scheduler::{JobScheduler, JobToRun, Job};
use tokio::time::Duration;


pub async fn set_reminder(cx: Cx) -> anyhow::Result<()> {
    // get user
    let user = cx.update.from().ok_or_else(|| anyhow!("No user found"))?;
    if user.username.is_none() {
        cx.reply_to("You must have a username to use this feature").await?;
        return Ok(());
    }

    let message = cx.update.reply_to_message();
    if message.is_none() {
        cx.reply_to("You must use this in reply to a message").await?;
        return Ok(());
    }
    let message_id = message.unwrap().id;

    let (_, args) = utils::extract_user_and_text(&cx, pool).await;
    if args.is_none() {
        cx.reply_to("You need to specify a duration in d/h/m/s (days, hours, minutes, seconds)").await?;
        return Ok(());
    }

    // We only care about the first argument
    let time_period = args.unwrap().split_whitespace().next();

    let unit = time_period.parse::<UnitOfTime>();
    if unit.is_err() {
        cx.reply_to("failed to get specified time; expected one of d/h/m/s (days, hours, minutes, seconds)").await?;
        return Ok(());
    }

    // convert to seconds
    let time_seconds = utils::extract_time(unit.as_ref().unwrap());
    schedule(cx, time_seconds, user.id, message_id).await?;

    Ok(())
}

async fn schedule(cx: Cx, duration: u64, chat_id: i64, message_id: i32) {
    let message_url = format!("https://t.me/{}/{}", chat_id, message_id);

    let reminder_job = Job::new_one_shot(Duration::from_secs(duration), |_, _| {
        cx.answer(format!("Here's your reminder.\nMessage: {}", message_url)).await?;
    }).unwrap_or_else({ cx.reply_to("Error creating a reminder") });

    let mut sched = JobScheduler::new();
    sched.add(reminder_job);
    sched.start();
}