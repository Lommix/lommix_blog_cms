use super::{Crud, SchemaDown, SchemaUp};
use rusqlite::{
    params,
    types::{FromSql, ToSqlOutput},
    ToSql,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum ParagraphType {
    Markdown,
    Html,
}

impl FromSql for ParagraphType {
    fn column_result(value: rusqlite::types::ValueRef) -> rusqlite::types::FromSqlResult<Self> {
        match value.as_str()? {
            "markdown" => Ok(ParagraphType::Markdown),
            "html" => Ok(ParagraphType::Html),
            _ => Ok(ParagraphType::Html),
        }
    }
}

impl ToSql for ParagraphType {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        match self {
            ParagraphType::Markdown => Ok("markdown".into()),
            ParagraphType::Html => Ok("html".into()),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Paragraph {
    pub id: Option<i64>,
    pub article_id: i64,
    pub title: String,
    pub description: String,
    pub paragraph_type: ParagraphType,
    pub position: i64,
    pub content: String,
    pub rendered: Option<String>,
}

impl Paragraph {
    pub fn find_by_article_id(
        article_id: i64,
        con: &rusqlite::Connection,
    ) -> Result<Vec<Self>, rusqlite::Error> {
        let mut stmt = con.prepare(
            "SELECT id, article_id, title, description, paragraph_type, position, content FROM paragraph WHERE article_id = ?"
        )?;
        let mut rows = stmt.query([&article_id])?;
        let mut paragraphs = Vec::new();
        while let Some(row) = rows.next()? {
            let mut para = Paragraph {
                id: row.get(0)?,
                article_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                paragraph_type: row.get(4)?,
                position: row.get(5)?,
                content: row.get(6)?,
                rendered: None,
            };

            match para.paragraph_type {
                ParagraphType::Markdown => para.rendered = Some(markdown::to_html(&para.content)),
                _ => (),
            };

            paragraphs.push(para);
        }

        Ok(paragraphs)
    }
}

impl SchemaUp for Paragraph {
    fn up(con: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        con.execute(
            "CREATE TABLE IF NOT EXISTS paragraph (
                id INTEGER PRIMARY KEY,
                article_id INTEGER,
                title TEXT,
                description TEXT,
                paragraph_type TEXT,
                position INTEGER,
                content TEXT
            );",
            (),
        )?;
        Ok(())
    }
}

impl Crud for Paragraph {
    fn find_all(con: &rusqlite::Connection) -> Result<Vec<Self>, rusqlite::Error> {
        let mut paragraphs = Vec::new();
        let mut stmt = con.prepare(
            "SELECT id, article_id, title, description, paragraph_type, position, content FROM paragraph",
        )?;
        let mut rows = stmt.query([])?;
        while let Some(row) = rows.next()? {
            let mut para = Paragraph {
                id: row.get(0)?,
                article_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                paragraph_type: row.get(4)?,
                position: row.get(5)?,
                content: row.get(6)?,
                rendered: None,
            };

            match para.paragraph_type {
                ParagraphType::Markdown => para.rendered = Some(markdown::to_html(&para.content)),
                _ => (),
            }

            paragraphs.push(para);
        }
        Ok(paragraphs)
    }

    fn find(id: i64, con: &rusqlite::Connection) -> Result<Self, rusqlite::Error> {
        let mut stmt = con.prepare(
            "SELECT id, article_id, title, description, paragraph_type, position, content FROM paragraph WHERE id = ?;"
        )?;
        let mut rows = stmt.query([&id])?;
        match rows.next()? {
            Some(row) => {
                let mut para = Paragraph {
                    id: row.get(0)?,
                    article_id: row.get(1)?,
                    title: row.get(2)?,
                    description: row.get(3)?,
                    paragraph_type: row.get(4)?,
                    position: row.get(5)?,
                    content: row.get(6)?,
                    rendered: None,
                };
                match para.paragraph_type {
                    ParagraphType::Markdown => para.rendered = Some(markdown::to_html(&para.content)),
                    _ => (),
                }
                Ok(para)
            }
            None => Err(rusqlite::Error::QueryReturnedNoRows),
        }
    }

    fn insert(&mut self, con: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        let mut stmt = con.prepare(
            "INSERT INTO paragraph (article_id, title, description, paragraph_type, position, content) VALUES (?, ?, ?, ?, ?, ?);",
        )?;
        stmt.execute(params![
            &self.article_id,
            &self.title,
            &self.description,
            &self.paragraph_type,
            &self.position,
            &self.content,
        ])?;
        self.id = Some(con.last_insert_rowid());
        Ok(())
    }

    fn update(&self, con: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        let mut stmt = con.prepare(
            "UPDATE paragraph SET article_id = ?, title = ?, description = ?, paragraph_type = ?, position = ?, content = ? WHERE id = ?;",
        )?;

        stmt.execute(params![
            &self.article_id,
            &self.title,
            &self.description,
            &self.paragraph_type,
            &self.position,
            &self.content,
            &self.id
        ])?;

        Ok(())
    }

    fn delete(id: i64, con: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
        let mut stmt = con.prepare("DELETE FROM paragraph WHERE id = ?")?;
        stmt.execute([&id])?;
        Ok(())
    }
}
