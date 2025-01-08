use dashmap::DashMap;
use pherris::analyzer::parser::Parser;
use pherris::lsp::lsp::Backend;
use pherris::lsp::state::State;
use std::sync::RwLock;
use tower_lsp::{LspService, Server};
use tracing::debug;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::Subscriber;

#[tokio::main]
async fn main() {
    let appender = tracing_appender::rolling::never(
        "/Users/miroslavzaprazny/personal/pherris/debug/",
        "log.txt",
    );
    let (writer, _guard) = tracing_appender::non_blocking(appender);
    let subscriber = Subscriber::builder()
        .with_writer(writer)
        .with_max_level(LevelFilter::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("to set subscriber");

    debug!("Starting lsp server");
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(|_| Backend {
        parser: RwLock::new(Parser::new().unwrap()),
        state: State::new(
            DashMap::default(),
            DashMap::default(),
            RwLock::new(String::from("")),
            DashMap::default(),
        ),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
