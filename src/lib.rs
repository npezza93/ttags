#[macro_use]
extern crate clap;

use std::error::Error;

pub mod cli;
pub mod config;
pub mod tag;
pub mod tagger;
pub mod lsp;
pub mod ruby;
pub mod javascript;
pub mod rust;

use config::Config;
use crate::tagger::Tagger;
use crate::lsp::Lsp;

pub struct App {}

impl App {
    pub fn run() -> Result<i32, Box<dyn Error>> {
        let config = Config::new();
        let mut tagger = Tagger::new(&config);

        if config.lsp {
            Lsp::run(&mut tagger)?;
        } else {
            tagger.run(&config.files);
        }

        Ok(0)
    }
}
