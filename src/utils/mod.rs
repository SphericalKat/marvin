pub mod perms;

use std::{fmt::Display, str::FromStr};

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
                .filter(|entity| matches!(entity.kind, MessageEntityKind::TextMention { user: _ }))
                .collect();

            // use first entity for extracting user
            if !filtered_entities.is_empty() {
                ent = Some(&entities[0]);
            }

            // if entity offset matches (command end/text start) then all is well
            if !entities.is_empty() && ent.is_some() {
                if ent.unwrap().offset == msg_text.len() - text_to_parse.len() {
                    ent = Some(&entities[0]);
                    user_id = match &ent.unwrap().kind {
                        MessageEntityKind::TextMention { user } => Some(user.id),
                        _ => None,
                    };
                    text = Some(msg_text[ent.unwrap().offset + ent.unwrap().length..].to_owned());
                }
            // args exist and first arg is a @ mention
            } else if !args.is_empty() && args[0].starts_with('@') {
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
            } else if !args.is_empty() {
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

pub enum UnitOfTime {
    Seconds(u64),
    Minutes(u64),
    Hours(u64),
    Days(u64),
}

impl FromStr for UnitOfTime {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        // attempt to split into two
        let split_s: Vec<_> = s.splitn(2, char::is_whitespace).collect();
        let parsed_num; // parsed number
        let u; // unit of time

        // user specified as a single slug, eg: '1m'
        if split_s.len() == 1
            && split_s[0].ends_with(&['h', 'm', 's', 'd'][..]) // check if suffixes are valid
            && split_s[0].len() >= 2
        // check if it's not _only_ a suffix
        {
            let mut time = split_s[0].to_owned(); // apparently only Strings can be popped
            let unit = time.pop().unwrap().to_string(); // pop suffix from string
            time = time.to_string(); // remaining part is time

            // attempt to parse time string
            parsed_num = match time.parse::<u64>() {
                Ok(n) => n,
                Err(_) => {
                    return Err("Allowed units: h, m, s, d");
                }
            };
            u = unit;
        // user specified with a whitespace, eg: '1 m'
        } else if split_s.len() == 2 {
            // attempt to parse time string
            parsed_num = match split_s[0].parse::<u64>() {
                Ok(n) => n,
                Err(_) => {
                    return Err("Allowed units: h, m, s, d");
                }
            };

            // second half is the unit
            u = split_s[1].to_owned()
        } else {
            // user is a dumbass
            return Err("Allowed units: h, m, s, d");
        }

        // attempt to match suffixes to units of time
        match &u as &str {
            "h" | "hours" => Ok(UnitOfTime::Hours(parsed_num)),
            "m" | "minutes" => Ok(UnitOfTime::Minutes(parsed_num)),
            "s" | "seconds" => Ok(UnitOfTime::Seconds(parsed_num)),
            "d" | "days" => Ok(UnitOfTime::Days(parsed_num)),
            _ => Err("Allowed units: h, m, s, d"), // user is a dumbass
        }
    }
}

// useful for formatting while sending
impl Display for UnitOfTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnitOfTime::Seconds(t) => write!(f, "{} second(s)", t),
            UnitOfTime::Minutes(t) => write!(f, "{} minute(s)", t),
            UnitOfTime::Hours(t) => write!(f, "{} hour(s)", t),
            UnitOfTime::Days(t) => write!(f, "{} day(s)", t),
        }
    }
}

// get time in seconds from unit and number of units
pub fn extract_time(unit: &UnitOfTime) -> u64 {
    match unit {
        UnitOfTime::Hours(t) => t * 3600,
        UnitOfTime::Minutes(t) => t * 60,
        UnitOfTime::Seconds(t) => *t,
        UnitOfTime::Days(t) => t * 3600 * 24,
    }
}

pub enum PinMode {
    Silent,
    Loud,
}

impl PinMode {
    pub fn is_silent(&self) -> bool {
        match self {
            PinMode::Silent => true,
            PinMode::Loud => false,
        }
    }
}

impl FromStr for PinMode {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        Ok(match s {
            "notify" | "loud" | "violent" => PinMode::Loud,
            _ => PinMode::Silent,
        })
    }
}
