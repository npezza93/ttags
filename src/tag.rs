use crate::config::Config;
use std::path::Path;
use sugar_path::SugarPath;

#[derive(Clone)]
pub struct Tag {
    pub name: String,
    pub filename: String,
    pub row: usize,
    pub kind: String,
}

impl Tag {
    pub fn parse(tag: &str) -> Self {
        let parts = tag.trim().split('\t').collect::<Vec<&str>>();
        let name = parts[0].to_string();
        let filename = Path::new(parts[1]).resolve().into_os_string().into_string().unwrap();
        let row = parts[2];
        let kind = parts[3].to_string();

        Tag { name, filename,
              row: row[0..row.len() - 2].parse::<usize>().unwrap(), kind }
    }

    pub fn new(name: &str, filename: &str, row: usize, kind: &str) -> Self {
        Tag {
            name: name.to_owned(),
            filename: filename.to_owned(),
            row,
            kind: kind.to_owned()
        }
    }

    pub fn as_bytes(&self, config: &Config) -> Vec<u8> {
        format!("{}\t{}\t{};\"\t{}\n",
            self.name, config.path_relative_to_file(&self.filename), self.row, self.kind).as_bytes().to_vec()
    }
}
