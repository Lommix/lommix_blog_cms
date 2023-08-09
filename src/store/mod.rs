mod blog;

pub fn schema_up(con: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    Ok(())
}


pub trait Find
where
    Self: Sized,
{
    fn find(id: i64) -> Result<Self, rusqlite::Error>;
}

pub trait FindAll
where
    Self: Sized,
{
    fn findAll() -> Result<Vec<Self>, rusqlite::Error>;
}

pub trait Insert
where
    Self: Sized,
{
    fn insert(&self) -> Result<(), rusqlite::Error>;
}

pub trait Upate
where
    Self: Sized,
{
    fn update(&self) -> Result<(), rusqlite::Error>;
}
