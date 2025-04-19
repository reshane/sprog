fn main() {
    let connection = sqlite::open("test.db").unwrap();
    let query = "
        DROP TABLE IF EXISTS comments;

        DROP TABLE IF EXISTS notes;

        DROP TABLE IF EXISTS users;

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

        CREATE TABLE comments (
            id integer primary key autoincrement,
            owner_id integer,
            note_id integer,
            contents text,
            foreign key(owner_id) references users(id),
            foreign key(note_id) references notes(id));

        CREATE TABLE punches (
            id integer primary key autoincrement,
            owner_id integer,
            geo text,
            foreign key(owner_id) references users(id));
    ";
    connection.execute(query).unwrap();
}
