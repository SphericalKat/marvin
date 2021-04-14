use barrel::{Migration, types, backend::Pg};

pub fn migrate() {
    let mut m = Migration::new();

    // create chats table
    m.create_table_if_not_exists("chats", |t| {
        t.add_column("chat_id", types::text().primary(true));
        t.add_column("chat_name", types::text().indexed(true));
    });

    println!("{}", m.make::<Pg>())
}