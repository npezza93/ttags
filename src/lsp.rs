use std::error::Error;
use crate::tagger::Tagger;

use lsp_types::{TextDocumentSyncKind, TextDocumentSyncCapability, TextDocumentSyncOptions, ServerCapabilities};
use lsp_server::{Connection, Message};

pub struct Lsp {
}

impl Lsp {
    pub fn run(tagger: &'_ mut Tagger) -> Result<i32, Box<dyn Error>> {
        let (connection, io_threads) = Connection::stdio();

        connection.initialize(Self::server_capabilities())?;

        for msg in &connection.receiver {
            match msg {
                Message::Request(req) => {
                    if connection.handle_shutdown(&req)? {
                        return Ok(0);
                    }
                }
                Message::Response(_resp) => {}
                Message::Notification(notification) => {
                    let uri = &notification.params["textDocument"]["uri"].as_str().unwrap();
                    let contents = notification.params["contentChanges"][0]["text"].as_str().unwrap().as_bytes();
                    let tags = tagger.parse(&uri[7..uri.len()], contents);

                    tagger.write(tags);
                }
            }
        }

        io_threads.join()?;

        Ok(0)
    }

    fn server_capabilities() -> serde_json::Value {
        let capabilities = ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Options(TextDocumentSyncOptions {
                open_close: None,
                change: Some(TextDocumentSyncKind::FULL),
                will_save: None,
                will_save_wait_until: None,
                save: None,
            })),
            ..Default::default()
        };

        serde_json::to_value(capabilities).unwrap()
    }
}
