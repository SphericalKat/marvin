use sqlx::{Pool, Postgres};
use teloxide::{
    prelude::Requester,
    types::{MessageEntity, MessageEntityKind},
};

use crate::{repo::users, Cx};

pub fn id_from_reply(cx: &Cx) -> (Option<i64>, Option<String>) {
    let prev_message = cx.update.reply_to_message();
    if prev_message.is_none() {
        return (None, None);
    }

    if let Some(user) = prev_message.unwrap().from() {
        if let Some(msg_text) = prev_message.unwrap().text() {
            let res: Vec<_> = msg_text.splitn(2, char::is_whitespace).collect();
            if res.len() < 2 {
                return (Some(user.id), Some("".to_owned()));
            }
            return (Some(user.id), Some(res[1].to_owned()));
        }
    }

    return (None, None);
}

pub async fn extract_user_and_text(
    cx: &Cx,
    pool: &Pool<Postgres>,
) -> (Option<i64>, Option<String>) {
    let prev_message = cx.update.reply_to_message();

    if let Some(msg_text) = cx.update.text() {
        let split_text: Vec<_> = msg_text.splitn(2, char::is_whitespace).collect();

        if split_text.len() < 2 {
            return id_from_reply(cx);
        }
        let text_to_parse = split_text[1];
        let args: Vec<_> = text_to_parse.split_whitespace().collect();
        let mut user_id: Option<i64> = None;

        let mut text: Option<String> = None;

        let mut ent: Option<&MessageEntity> = None;

        if let Some(entities) = cx.update.entities() {
            let filtered_entities: Vec<_> = entities
                .iter()
                .filter(|entity| match entity.kind {
                    MessageEntityKind::TextMention { user: _ } => true,
                    _ => false,
                })
                .collect();

            if filtered_entities.len() > 0 {
                ent = Some(&entities[0]);
            }

            // if entty offset matches (command end/text start) then all is well
            if entities.len() != 0 && ent.is_some() {
                if ent.unwrap().offset == msg_text.len() - text_to_parse.len() {
                    ent = Some(&entities[0]);
                    user_id = match &ent.unwrap().kind {
                        MessageEntityKind::TextMention { user } => Some(user.id),
                        _ => None,
                    };
                    text = Some(msg_text[ent.unwrap().offset + ent.unwrap().length..].to_owned());
                }

            } else if args.len() >= 1 && args[0].chars().nth(0) == Some('@') {
                let user_name = args[0];
                let res =
                    users::get_user(None, Some(user_name.to_lowercase().replace("@", "")), pool)
                        .await;
                if res.is_ok() {
                    user_id = Some(res.unwrap().user_id);
                    let split: Vec<_> = msg_text.splitn(3, char::is_whitespace).collect();
                    if split.len() >= 3 {
                        text = Some(split[2].to_owned());
                    }
                } else {
                    cx.reply_to("Could not find a user by this name; are you sure I've seen them before?").await.ok();
                    return (None, None);
                }
            } else if args.len() >= 1 {
                if let Ok(id) = args[0].parse::<i64>() {
                    user_id = Some(id);
                    let res: Vec<_> = msg_text.splitn(3, char::is_whitespace).collect();
                    if res.len() >= 3 {
                        text = Some(res[2].to_owned());
                    }
                }
            } else if prev_message.is_some() {
                (user_id, text) = id_from_reply(&cx);
            } else {
                return (None, None);
            }

            if let Some(id) = user_id {
                match cx.requester.get_chat(id).await {
                    Ok(_) => {}
                    Err(_) => {
                        cx.reply_to("I don't seem to have interacted with this user before - please forward a message from them to give me control! (like a voodoo doll, I need a piece of them to be able to execute certain commands...)").await.ok();
                        return (None, None);
                    }
                }
            }
        }
        return (user_id, text);
    }
    (None, None)
}
