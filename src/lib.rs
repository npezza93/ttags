#[macro_use]
extern crate clap;
extern crate tower_lsp;

use std::error::Error;
use tower_lsp::{LspService, Server};

pub mod config;
pub mod lsp;
pub mod tag;
pub mod tagger;
pub mod ruby;
pub mod javascript;
pub mod rust;

use config::Config;
use crate::tagger::Tagger;
use lsp::Backend;

pub struct App {}

impl App {
    #[tokio::main]
    pub async fn run() -> Result<i32, Box<dyn Error>> {
        let config = Config::new();

        if config.lsp {
            let stdin = tokio::io::stdin();
            let stdout = tokio::io::stdout();

            let (service, socket) = LspService::new(|client| Backend { client });
            Server::new(stdin, stdout, socket).serve(service).await;
        } else {
            Tagger::run(&config, &config.files);
        }

        Ok(0)
    }
}
