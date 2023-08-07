use std::{fs, path::PathBuf};


pub struct Assets<'source> {
    pub templates: minijinja::Environment<'source>,
}


impl Assets<'_> {
    pub fn from_dir(dir: PathBuf) -> Result<Self, std::io::Error> {
        // let mut files = Vec::new();


        Ok(Self {
            templates: minijinja::Environment::new(),
        })
    }
}
