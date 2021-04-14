use barrel::{Migration, types, backend::Pg};

pub fn migration() -> String {
    let mut m = Migration::new();

    // create users table
    m.create_table_if_not_exists("users", |t| {
        t.add_column("user_id", types::text().primary(true));
        t.add_column("user_name", types::text().indexed(true).nullable(true));
    });

     // create chats table
     m.create_table_if_not_exists("chats", |t| {
        t.add_column("chat_id", types::text().primary(true));
        t.add_column("chat_name", types::text().indexed(true));
    });


    m.make::<Pg>()
}