pub mod articles;
pub mod paragraphs;

pub fn schema_up(con: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    Ok(())
}

pub trait Crud
where
    Self: Sized,
{
    fn find(id: i64, con: &rusqlite::Connection) -> Result<Self, rusqlite::Error>;
    fn find_all(con: &rusqlite::Connection) -> Result<Vec<Self>, rusqlite::Error>;
    fn insert(&mut self, con: &rusqlite::Connection) -> Result<(), rusqlite::Error>;
    fn update(&self, con: &rusqlite::Connection) -> Result<(), rusqlite::Error>;
    fn delete(id: i64, con: &rusqlite::Connection) -> Result<(), rusqlite::Error>;
}

pub trait SchemaUp
where
    Self: Sized,
{
    fn up(con: &rusqlite::Connection) -> Result<(), rusqlite::Error>;
}

pub trait SchemaDown
where
    Self: Sized,
{
    fn down(con: &rusqlite::Connection) -> Result<(), rusqlite::Error>;
}
