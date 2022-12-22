use crate::config::Config;

pub struct Tag {
    pub name: String,
    pub filename: String,
    pub row: usize,
    pub kind: String,
}

impl Tag {
    pub fn new(name: &str, filename: &str, row: usize, kind: &str) -> Self {
        Tag {
            name: name.to_owned(),
            filename: filename.to_owned(),
            row: row + 1,
            kind: kind.to_owned()
        }
    }

    pub fn as_bytes(self, config: &Config) -> Vec<u8> {
        format!("{}\t{}\t{};\"\t{}\n",
            self.name, config.path_relative_to_file(&self.filename), self.row, self.kind).as_bytes().to_vec()
    }
}
