use std::error::Error;
use crate::tagger::Tagger;

use lsp_types::{TextDocumentSyncKind, TextDocumentSyncCapability};
use lsp_types::{TextDocumentSyncSaveOptions, TextDocumentSyncOptions};
use lsp_types::ServerCapabilities;
use lsp_server::{Connection, Message};

pub struct Lsp {
}

impl Lsp {
    pub fn run(tagger: &'_ mut Tagger) -> Result<i32, Box<dyn Error>> {
        let (connection, io_threads) = Connection::stdio();

        eprintln!("starting ttags LSP server");
        connection.initialize(Self::server_capabilities())?;

        for msg in &connection.receiver {
            eprintln!("got msg: {msg:?}");

            match msg {
                Message::Request(req) => {

                    if connection.handle_shutdown(&req)? {
                        return Ok(0);
                    }
                }
                Message::Response(_resp) => {}
                Message::Notification(notification) => {
                    let uri = &notification.params["textDocument"]["uri"].as_str().unwrap();
                    let tags = tagger.read_and_parse(&uri[7..uri.len()]);

                    tagger.write(tags);
                }
            }
        }

        io_threads.join()?;
        eprintln!("shutting down ttags server");

        Ok(0)
    }

    fn server_capabilities() -> serde_json::Value {
        let capabilities = ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Options(TextDocumentSyncOptions {
                open_close: None,
                change: Some(TextDocumentSyncKind::NONE),
                will_save: None,
                will_save_wait_until: None,
                save: Some(TextDocumentSyncSaveOptions::Supported(true)),
            })),
            ..Default::default()
        };

        serde_json::to_value(capabilities).unwrap()
    }
}
