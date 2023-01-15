#[macro_use]
extern crate clap;

use std::error::Error;

pub mod config;
pub mod tag;
pub mod tagger;
pub mod ruby;
pub mod javascript;
pub mod rust;

use config::Config;
use crate::tagger::Tagger;

pub struct App {}

impl App {
    pub fn run() -> Result<i32, Box<dyn Error>> {
        let config = Config::new();

        if config.lsp {
            println!("lsp");
        } else {
            Tagger::run(&config, &config.files);
        }

        Ok(0)
    }
}
