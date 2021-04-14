use barrel::{Migration, types, backend::Pg};

pub fn migrate() {
    let mut m = Migration::new();

    m.create_table_if_not_exists("users", |t| {
        t.add_column("user_id", types::text().primary(true));
        t.add_column("user_name", types::text().indexed(true).nullable(true));
    });

    println!("{}", m.make::<Pg>())
}