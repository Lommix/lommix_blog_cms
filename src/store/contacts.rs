use super::{Crud, SchemaUp};
use rusqlite::params;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ContactRequest {
    pub id: Option<i64>,
    pub created: i64,
    pub email: String,
    pub subject: String,
    pub message: String,
}

impl ContactRequest {
    pub fn count_all(con: &rusqlite::Connection) -> Result<i64, rusqlite::Error> {
        let mut stmt = con.prepare("SELECT COUNT(*) FROM contacts")?;
        let mut rows = stmt.query([])?;
        match rows.next()? {
            Some(row) => Ok(row.get(0)?),
            None => Ok(0),
        }
    }

    pub fn find_all_orderd(
        limit: i64,
        offset: i64,
        con: &rusqlite::Connection,
    ) -> Result<Vec<ContactRequest>, rusqlite::Error> {
        let mut stmt =
            con.prepare("SELECT * FROM contacts ORDER BY created DESC LIMIT ? OFFSET ?")?;
        let mut rows = stmt.query([limit, offset])?;
        let mut result = Vec::new();
        while let Some(row) = rows.next()? {
            result.push(ContactRequest {
                id: row.get(0)?,
                created: row.get(1)?,
                email: row.get(2)?,
                subject: row.get(3)?,
                message: row.get(4)?,
            })
        }
        Ok(result)
    }
}

impl SchemaUp for ContactRequest {
    fn up(con: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        con.execute(
            "CREATE TABLE IF NOT EXISTS contacts (
                id INTEGER PRIMARY KEY,
                created INTEGER,
                email TEXT,
                subject TEXT,
                message TEXT
            )",
            (),
        )?;
        Ok(())
    }
}

impl Crud for ContactRequest {
    fn find(id: i64, con: &rusqlite::Connection) -> Result<Self, rusqlite::Error> {
        let mut stmt = con.prepare("SELECT * FROM contacts WHERE id = ?")?;
        let mut rows = stmt.query([&id])?;
        match rows.next()? {
            Some(row) => Ok(ContactRequest {
                id: row.get(0)?,
                created: row.get(1)?,
                email: row.get(2)?,
                subject: row.get(3)?,
                message: row.get(4)?,
            }),
            None => Err(rusqlite::Error::QueryReturnedNoRows),
        }
    }

    fn find_all(con: &rusqlite::Connection) -> Result<Vec<Self>, rusqlite::Error> {
        let mut stmt = con.prepare("SELECT * FROM contacts")?;
        let mut rows = stmt.query([])?;
        let mut contacts = Vec::new();
        while let Some(row) = rows.next()? {
            contacts.push(ContactRequest {
                id: row.get(0)?,
                created: row.get(1)?,
                email: row.get(2)?,
                subject: row.get(3)?,
                message: row.get(4)?,
            })
        }
        Ok(contacts)
    }

    fn insert(&mut self, con: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        let mut stmt = con.prepare(
            "INSERT INTO contacts (email ,created, subject, message) VALUES (?, ?, ?, ?)",
        )?;
        stmt.execute(params![
            &self.email,
            &self.created,
            &self.subject,
            &self.message
        ])?;
        self.id = Some(con.last_insert_rowid());
        Ok(())
    }

    fn update(&self, con: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        let mut stmt =
            con.prepare("UPDATE contacts SET email = ?, subject = ?, message = ? WHERE id = ?")?;
        stmt.execute(params![
            &self.email,
            &self.subject,
            &self.message,
            &self.id.ok_or(rusqlite::Error::InvalidQuery)?,
        ])?;
        Ok(())
    }

    fn delete(id: i64, con: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        let mut stmt = con.prepare("DELETE FROM contacts WHERE id = ?")?;
        stmt.execute([&id])?;
        Ok(())
    }
}
