use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::Subscriber;
use tracing::debug;
use tree_sitter::Tree;
use dashmap::DashMap;

#[derive(Debug)]
struct Backend {
    client: Client,
    ast_map: DashMap<Url, Tree>
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions{
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL), // dont we want incremental
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                        will_save: Some(true), //idk what this does
                        will_save_wait_until: Some(true) //idk what this does
                    }
                )),
                definition_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "server initialized!")
            .await;
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        debug!("Goto definition params: {:?}", params);
        let url = Url::parse("/Users/miroslavzaprazny/workspace/services").expect("to parse url");
        let range = Range::new(Position::new(0, 0), Position::new(0,0));
        let location = Location::new(url, range);
        let response = GotoDefinitionResponse::Scalar(location);

        Ok(Some(response))
    }

    /// The [`textDocument/didOpen`] notification is sent from the client to the server to signal
    /// that a new text document has been opened by the client.
    ///
    /// [`textDocument/didOpen`]: https://microsoft.github.io/language-server-protocol/specification#textDocument_didOpen
    ///
    /// The document's truth is now managed by the client and the server must not try to read the
    /// document’s truth using the document's URI. "Open" in this sense means it is managed by the
    /// client. It doesn't necessarily mean that its content is presented in an editor.
    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        debug!("Got a textDocument/didOpen notification");

        let lang = tree_sitter_php::LANGUAGE_PHP;
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&lang.into()).expect("to set lang");

        //should we panic?
        let tree = parser.parse(params.text_document.text, None).expect("to parse file");
        self.ast_map.insert(params.text_document.uri, tree).expect("to insert tree");
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let appender = tracing_appender::rolling::never("/Users/miroslavzaprazny/personal/pherris/debug/", "log.txt");
    let (writer, _guard) = tracing_appender::non_blocking(appender);
    let subscriber = Subscriber::builder()
        .with_writer(writer)
        .with_max_level(LevelFilter::DEBUG)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("to set subscriber");

    debug!("Starting lsp server");
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(|client| Backend { client, ast_map: DashMap::default() });
    Server::new(stdin, stdout, socket).serve(service).await;
}
