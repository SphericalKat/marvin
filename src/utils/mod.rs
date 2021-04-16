use sqlx::{Pool, Postgres};
use teloxide::{
    prelude::Requester,
    types::{MessageEntity, MessageEntityKind},
};

use crate::{repo::users, Cx};

pub fn id_from_reply(cx: &Cx) -> (Option<i64>, Option<String>) {
    // check for reply
    let prev_message = cx.update.reply_to_message();
    if prev_message.is_none() {
        return (None, None);
    }

    // if can get user from replied-to message
    if let Some(user) = prev_message.unwrap().from() {
        // if quoted message has some text
        if let Some(msg_text) = prev_message.unwrap().text() {
            // split into args
            let res: Vec<_> = msg_text.splitn(2, char::is_whitespace).collect();

            // no args, return only user ID
            if res.len() < 2 {
                return (Some(user.id), Some("".to_owned()));
            }

            // return user ID and text
            return (Some(user.id), Some(res[1].to_owned()));
        }
    }

    // nothing found, bail
    (None, None)
}

pub async fn extract_user_and_text(
    cx: &Cx,
    pool: &Pool<Postgres>,
) -> (Option<i64>, Option<String>) {
    if let Some(msg_text) = cx.update.text() {
        // split into command and args
        let split_text: Vec<_> = msg_text.splitn(2, char::is_whitespace).collect();

        // only command exists, return ID from reply
        if split_text.len() < 2 {
            return id_from_reply(cx);
        }

        // parse second part of split
        let text_to_parse = split_text[1];
        let args: Vec<_> = text_to_parse.split_whitespace().collect(); // split into vec of strings

        let mut user_id: Option<i64> = None; // user id to return
        let mut text: Option<String> = None; // text to return
        let mut ent: Option<&MessageEntity> = None; // mentioned entity in message

        // if entities exist in message
        if let Some(entities) = cx.update.entities() {
            // filter out only text mention entities
            let filtered_entities: Vec<_> = entities
                .iter()
                .filter(|entity| match entity.kind {
                    MessageEntityKind::TextMention { user: _ } => true,
                    _ => false,
                })
                .collect();

            // use first entity for extracting user
            if filtered_entities.len() > 0 {
                ent = Some(&entities[0]);
            }

            // if entity offset matches (command end/text start) then all is well
            if entities.len() != 0 && ent.is_some() {
                if ent.unwrap().offset == msg_text.len() - text_to_parse.len() {
                    ent = Some(&entities[0]);
                    user_id = match &ent.unwrap().kind {
                        MessageEntityKind::TextMention { user } => Some(user.id),
                        _ => None,
                    };
                    text = Some(msg_text[ent.unwrap().offset + ent.unwrap().length..].to_owned());
                }
            // args exist and first arg is a @ mention
            } else if args.len() >= 1 && args[0].chars().nth(0) == Some('@') {
                let user_name = args[0];
                let res =
                    users::get_user(None, Some(user_name.to_string().replace("@", "")), pool).await;
                if res.is_ok() {
                    user_id = Some(res.unwrap().user_id);
                    let split: Vec<_> = msg_text.splitn(3, char::is_whitespace).collect();
                    if split.len() >= 3 {
                        text = Some(split[2].to_owned());
                    }
                } else {
                    cx.reply_to(
                        "Could not find a user by this name; are you sure I've seen them before?",
                    )
                    .await
                    .ok();
                    return (None, None);
                }
            // check if first argument is a user ID
            } else if args.len() >= 1 {
                if let Ok(id) = args[0].parse::<i64>() {
                    user_id = Some(id);
                    let res: Vec<_> = msg_text.splitn(3, char::is_whitespace).collect();
                    if res.len() >= 3 {
                        text = Some(res[2].to_owned());
                    }
                }
            // check if command is a reply to message
            } else if cx.update.reply_to_message().is_some() {
                (user_id, text) = id_from_reply(&cx);
            } else {
                // nothing satisfied, bail
                return (None, None);
            }

            // check if bot has interacted with this user before
            if let Some(id) = user_id {
                match cx.requester.get_chat(id).await {
                    Ok(_) => {}
                    Err(_) => {
                        // haven't seen this user, bail
                        cx.reply_to("I don't seem to have interacted with this user before - please forward a message from them to give me control! (like a voodoo doll, I need a piece of them to be able to execute certain commands...)").await.ok();
                        return (None, None);
                    }
                }
            }
        }

        // return user ID and extracted text
        return (user_id, text);
    }

    // nothing found, bail
    (None, None)
}
