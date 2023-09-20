use std::collections::HashMap;

use rusqlite::{
    params,
    types::{FromSql, ToSqlOutput},
    ToSql,
};
use serde::{Deserialize, Serialize};

use super::{Crud, SchemaUp};

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct ArticleViews {
    data: HashMap<i64, i64>,
}

impl ArticleViews {
    pub fn add(&mut self, article_id: i64) {
        *self.data.entry(article_id).or_insert(0) += 1
    }
}

impl ToSql for ArticleViews {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        let string = serde_json::to_string(self).map_err(|err| rusqlite::Error::InvalidQuery)?;
        Ok(string.into())
    }
}

impl FromSql for ArticleViews {
    fn column_result(value: rusqlite::types::ValueRef) -> rusqlite::types::FromSqlResult<Self> {
        serde_json::from_str::<ArticleViews>(value.as_str()?)
            .map_err(|er| rusqlite::types::FromSqlError::InvalidType)
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Stats {
    pub date: i64,
    pub home_views: i64,
    pub donate_views: i64,
    pub about_views: i64,
    pub article_views: ArticleViews,
}

impl Stats {
    pub fn find_or_create_today(con: &rusqlite::Connection) -> Result<Self, rusqlite::Error> {
        let now = chrono::offset::Local::now()
            .date_naive()
            .and_time(chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap())
            .timestamp();
        match Self::find(now, con) {
            Ok(stats) => Ok(stats),
            Err(_) => {
                let mut stats = Self {
                    date: now,
                    ..Default::default()
                };
                stats.insert(con)?;
                Ok(stats)
            }
        }
    }

    pub fn get_last_days(
        days: i64,
        con: &rusqlite::Connection,
    ) -> Result<Vec<Self>, rusqlite::Error> {
        let mut stmt = con.prepare("SELECT * FROM stats ORDER BY date DESC LIMIT ?")?;
        let mut rows = stmt.query([days])?;
        let mut result = Vec::new();

        while let Some(row) = rows.next()? {
            result.push(Stats {
                date: row.get(0)?,
                home_views: row.get(1)?,
                about_views: row.get(2)?,
                donate_views: row.get(3)?,
                article_views: row.get(4)?,
            });
        }
        Ok(result)
    }
}

impl SchemaUp for Stats {
    fn up(con: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        con.execute(
            "CREATE TABLE IF NOT EXISTS stats (
            date INTEGER PRIMARY KEY,
            home_views INTEGER DEFAULT 0,
            about_views INTEGER DEFAULT 0,
            donate_views INTEGER DEFAULT 0,
            article_views TEXT
            );",
            (),
        )?;
        Ok(())
    }
}

impl Crud for Stats {
    fn find(date: i64, con: &rusqlite::Connection) -> Result<Self, rusqlite::Error> {
        let mut stmt = con.prepare("SELECT * FROM stats WHERE date = ?")?;
        let mut rows = stmt.query([&date])?;
        if let Some(row) = rows.next()? {
            return Ok(Stats {
                date: row.get(0)?,
                home_views: row.get(1)?,
                about_views: row.get(2)?,
                donate_views: row.get(3)?,
                article_views: row.get(4)?,
            });
        }
        Err(rusqlite::Error::QueryReturnedNoRows)
    }

    fn find_all(con: &rusqlite::Connection) -> Result<Vec<Self>, rusqlite::Error> {
        let mut stmt = con.prepare("SELECT * FROM stats")?;
        let mut rows = stmt.query([])?;
        let mut result = Vec::new();
        while let Some(row) = rows.next()? {
            result.push(Stats {
                date: row.get(0)?,
                home_views: row.get(1)?,
                about_views: row.get(2)?,
                donate_views: row.get(3)?,
                article_views: row.get(4)?,
            })
        }
        Ok(result)
    }

    fn insert(&mut self, con: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        let mut stmt = con.prepare(
            "INSERT INTO stats (date, home_views, about_views, donate_views, article_views) VALUES (?, ?, ?, ?, ?)",
        )?;
        stmt.execute(params![
            &self.date,
            &self.home_views,
            &self.about_views,
            &self.donate_views,
            &self.article_views,
        ])?;

        Ok(())
    }

    fn update(&self, con: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        let mut stmt = con.prepare(
            "UPDATE stats SET home_views = ?, about_views = ?, donate_views = ?, article_views = ?",
        )?;
        stmt.execute(params![
            &self.home_views,
            &self.about_views,
            &self.donate_views,
            &self.article_views,
        ])?;
        Ok(())
    }

    fn delete(date: i64, con: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        let mut stmt = con.prepare("DELETE FROM stats WHERE date = ?")?;
        stmt.execute([&date])?;
        Ok(())
    }
}
