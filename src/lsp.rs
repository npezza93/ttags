use std::error::Error;

use lsp_types::{TextDocumentSyncKind, TextDocumentSyncCapability, TextDocumentSyncOptions, ServerCapabilities, InitializeParams};
use lsp_server::{Connection, Message};

pub fn run() -> Result<i32, Box<dyn Error>> {
    let (connection, io_threads) = Connection::stdio();

    let server_capabilities = serde_json::to_value(&ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Options(TextDocumentSyncOptions {
            open_close: None,
            change: Some(TextDocumentSyncKind::FULL),
            will_save: None,
            will_save_wait_until: None,
            save: None,
        })),
        ..Default::default()
    }).unwrap();
    let initialization_params = connection.initialize(server_capabilities)?;
    main_loop(connection, initialization_params)?;
    io_threads.join()?;

    Ok(0)
}

fn main_loop(
    connection: Connection,
    params: serde_json::Value,
) -> Result<(), Box<dyn Error>> {
    let _params: InitializeParams = serde_json::from_value(params).unwrap();
    for msg in &connection.receiver {
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
            }
            Message::Response(_resp) => {}
            Message::Notification(notification) => {
                let uri = &notification.params["textDocument"]["uri"].as_str().unwrap();
                let contents = &notification.params["contentChanges"][0]["text"].as_str().unwrap();
                eprintln!("got notification: {contents:?}");
            }
        }
    }

    Ok(())
}
