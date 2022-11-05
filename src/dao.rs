use rusqlite::{params, Connection, Error, Result};

pub struct Dao {
    conn: Connection,
}

#[derive(Debug)]
pub struct Entry {
    pub id: i32,
    pub name: String,
}

impl Dao {
    fn set_up(&self) -> Result<(), Error> {
        self.conn.execute(
            "CREATE TABLE person (
            id   INTEGER PRIMARY KEY,
            name TEXT NOT NULL
        )",
            (), // empty list of parameters.
        )?;

        self.conn
            .execute("INSERT INTO person (name) VALUES (?1)", params!["Frank"])?;
        self.conn
            .execute("INSERT INTO person (name) VALUES (?1)", params!["Peter"])?;
        Ok(())
    }

    pub fn new_in_memory() -> Result<Self, Error> {
        let dao = Self {
            conn: Connection::open_in_memory()?,
        };
        dao.set_up()?;
        Ok(dao)
    }

    pub fn get_persons(&self) -> Result<Vec<Entry>, ()> {
        if let Ok(mut stmt) = self.conn.prepare("SELECT id, name FROM person") {
            let entry_iter = stmt.query_map([], |row| {
                Ok(Entry {
                    id: row.get(0)?,
                    name: row.get(1)?,
                })
            });

            if let Ok(entry_iter) = entry_iter {
                let mut ret_val: Vec<Entry> = Vec::new();
                for entry in entry_iter {
                    if let Ok(entry) = entry {
                        ret_val.push(entry);
                    }
                }
                return Ok(ret_val);
            }
        }
        Err(())
    }
}
