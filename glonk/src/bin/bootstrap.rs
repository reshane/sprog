fn main() {
    let connection = sqlite::open("test.db").unwrap();
    let query = "
        CREATE TABLE users (
            id integer primary key autoincrement,
            guid text not null,
            name text,
            email text,
            picture text);

        CREATE TABLE notes (
            id integer primary key autoincrement,
            owner_id integer,
            contents text,
            foreign key(owner_id) references users(id));
    ";
    connection.execute(query).unwrap();
}
