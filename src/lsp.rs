use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};
use std::io::{self, Write, BufWriter, BufReader, Read};
use std::fs::{File, OpenOptions};

#[derive(Debug)]
pub struct Backend {
    pub client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        OpenOptions::new().write(true).append(true).open("/Users/nick/Documents/ttags/logfile").unwrap().write_all(b"A new line!");
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                 ..ServerCapabilities::default()
            }
        })

    }

    async fn initialized(&self, _: InitializedParams) {
        OpenOptions::new().write(true).append(true).open("/Users/nick/Documents/ttags/logfile").unwrap().write_all(b"A new line!");
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file opened!")
            .await;
        self.on_change(params.text_document.uri).await
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        println!("{:?}", params);
        self.on_change(params.text_document.uri).await
    }
}

impl Backend {
    async fn on_change(&self, uri: Url) {
        self.client
            .log_message(MessageType::INFO, "on changeeee")
            .await;
    }
}
