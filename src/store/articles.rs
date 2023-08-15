use super::paragraphs::Paragraph;
use super::{Crud, SchemaDown, SchemaUp};
use axum::async_trait;
use axum::extract::{FromRequest, FromRequestParts};
use axum::http::Request;
use rusqlite::params;
use rusqlite::{
    types::{FromSql, ToSqlOutput},
    ToSql,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Article {
    pub id: Option<i64>,
    pub title: String,
    pub teaser: String,
    pub cover: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub published: bool,
    pub paragraphs: Option<Vec<Paragraph>>,
}

impl Article {
    pub fn new(title: String) -> Self {
        let now = chrono::offset::Local::now().timestamp();
        Article {
            id: None,
            title,
            teaser: "".to_string(),
            cover: "".to_string(),
            created_at: now,
            updated_at: now,
            published: false,
            paragraphs: None,
        }
    }
}

impl SchemaUp for Article {
    fn up(con: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        con.execute(
            "CREATE TABLE IF NOT EXISTS article (
id INTEGER PRIMARY KEY,
title TEXT,
teaser TEXT,
cover TEXT,
created_at INTEGER,
updated_at INTEGER,
published BOOLEAN
);",
            (),
        )?;
        Ok(())
    }
}

impl Crud for Article {
    fn find_all(con: &rusqlite::Connection) -> Result<Vec<Self>, rusqlite::Error> {
        let mut articles = Vec::new();
        let mut stmt = con.prepare(
            "SELECT id, title, teaser, cover, created_at, updated_at, published FROM article ORDER BY created_at DESC",
        )?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            articles.push(Article {
                id: row.get(0)?,
                title: row.get(1)?,
                teaser: row.get(2)?,
                cover: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
                published: row.get(6)?,
                paragraphs: None,
            })
        }

        Ok(articles)
    }
    fn insert(&mut self, con: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        let mut stmt = con.prepare(
            "INSERT INTO article (title, teaser, cover, created_at, updated_at, published) VALUES (?, ?, ?, ?, ?, ?)",
        )?;

        stmt.execute(params![
            &self.title,
            &self.teaser,
            &self.cover,
            &self.created_at,
            &self.updated_at,
            &self.published,
        ])?;

        self.id = Some(con.last_insert_rowid());

        Ok(())
    }
    fn find(id: i64, con: &rusqlite::Connection) -> Result<Self, rusqlite::Error> {
        let mut stmt = con.prepare(
            "SELECT id, title, teaser, cover, created_at, updated_at, published FROM article WHERE id = ?"
        )?;
        let mut rows = stmt.query([&id])?;
        match rows.next()? {
            Some(row) => Ok(Article {
                id: row.get(0)?,
                title: row.get(1)?,
                teaser: row.get(2)?,
                cover: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
                published: row.get(6)?,
                paragraphs: Paragraph::find_by_article_id(id, con).ok(),
            }),

            None => Err(rusqlite::Error::QueryReturnedNoRows),
        }
    }

    fn update(&self, con: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        let mut stmt = con.prepare(
            "UPDATE article SET title = ?, teaser = ?, cover = ?, created_at = ?, updated_at = ?, published = ? WHERE id = ?",
        )?;

        stmt.execute(params![
            &self.title,
            &self.teaser,
            &self.cover,
            &self.created_at,
            &self.updated_at,
            &self.published,
            &self.id
        ])?;

        Ok(())
    }

    fn delete(id: i64, con: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        let mut stmt = con.prepare("DELETE FROM article WHERE id = ?")?;
        stmt.execute([&id])?;
        Ok(())
    }
}
