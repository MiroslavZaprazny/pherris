use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::Subscriber;
use tracing::debug;

#[derive(Debug)]
struct Backend {
    client: Client,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
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
        let url = Url::parse("/Users/miroslavzaprazny/workspace/service").expect("to parse url");
        let range = Range::new(Position::new(0, 0), Position::new(0,0));
        let location = Location::new(url, range);
        let response = GotoDefinitionResponse::Scalar(location);

        Ok(Some(response))
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
    let (service, socket) = LspService::new(|client| Backend { client });
    Server::new(stdin, stdout, socket).serve(service).await;
}
