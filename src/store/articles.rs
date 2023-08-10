use super::paragraphs::Paragraph;
use super::{SchemaUp, SchemaDown, Crud};
use axum::extract::{FromRequest, FromRequestParts};
use axum::http::Request;
use rusqlite::{
    types::{FromSql, ToSqlOutput},
    ToSql,
};
use axum::async_trait;


#[derive(Debug)]
pub struct Article {
    pub id: Option<i64>,
    pub title: String,
    pub teaser: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
    pub published: bool,
    pub paragraphs: Option<Vec<Paragraph>>,
}

impl Article{
    pub fn new(title: String) -> Self {
        Article {
            id: None,
            title,
            teaser: "".to_string(),
            description: "".to_string(),
            created_at: "".to_string(),
            updated_at: "".to_string(),
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
description TEXT,
created_at TEXT,
updated_at TEXT,
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
            "SELECT id, title, teaser, description, created_at, updated_at, published FROM article",
        )?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            articles.push(Article {
                id: row.get(0)?,
                title: row.get(1)?,
                teaser: row.get(2)?,
                description: row.get(3)?,
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
            "INSERT INTO article (title, teaser, description, created_at, updated_at, published) VALUES (?, ?, ?, ?, ?, ?)",
        )?;

        stmt.execute(&[
            &self.title,
            &self.teaser,
            &self.description,
            &self.created_at,
            &self.updated_at,
            &self.published.to_string(),
        ])?;

        self.id = Some(con.last_insert_rowid());

        Ok(())
    }
    fn find(id: i64, con: &rusqlite::Connection) -> Result<Self, rusqlite::Error> {
        let mut stmt = con.prepare(
            "SELECT id, title, teaser, description, created_at, updated_at, published FROM article WHERE id = ?"
        )?;
        let mut rows = stmt.query(&[&id])?;
        match rows.next()? {
            Some(row) => Ok(Article {
                id: row.get(0)?,
                title: row.get(1)?,
                teaser: row.get(2)?,
                description: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
                published: row.get(6)?,
                paragraphs: None,
            }),
            None => Err(rusqlite::Error::QueryReturnedNoRows),
        }
    }

    fn update(&self, con: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        todo!()
    }

    fn delete(id: i64, con: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        todo!()
    }
}
