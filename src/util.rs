use std::path::PathBuf;

pub struct Util;

impl Util {
    pub fn load_files_rec(dir: PathBuf) -> Result<Vec<(String, PathBuf)>, std::io::Error> {
        let mut files = Vec::new();
        let dir = std::fs::read_dir(dir)?;

        for file in dir {
            let path = file?.path();
            if path.is_dir() {
                files.append(&mut Util::load_files_rec(path)?);
            } else if path.is_file() {
                match path.file_name() {
                    Some(name) => {
                        let file_name = name
                            .to_str()
                            .ok_or(std::io::Error::new(
                                std::io::ErrorKind::Other,
                                "no file name",
                            ))?
                            .to_string();
                        files.push((file_name.to_string(), path));
                    }
                    None => (),
                };
            }
        }
        Ok(files)
    }
}
