pub enum ParagraphType {
    Text,
    Image,
    Video,
    Wasm,
}

pub struct Article {
    pub id: i64,
    pub title: String,
    pub teaser: String,
    pub description: String,
    pub created_at: String,
    pub updated_at: String,
    pub published: bool,
    pub content: Option<Vec<Paragraph>>,
}

pub struct Paragraph {
    pub id: i64,
    pub article_id: i64,
    pub title: String,
    pub description: String,
    pub paragraph_type: ParagraphType,
}
