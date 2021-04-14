pub mod chats;
mod users;

pub fn migrate() {
    users::migrate();
    chats::migrate();
}
