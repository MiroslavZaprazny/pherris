use crate::analyzer::composer::load_autoload_class_map;
use crate::analyzer::parser::Parser;
use crate::handlers::notification::handle_did_open;
use crate::handlers::request::handle_go_to_definition;
use std::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::LanguageServer;

use super::state::State;

pub struct Backend {
    pub parser: RwLock<Parser>,
    pub state: State,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        if let Some(root_uri) = params.root_uri {
            let mut guard = self.state.root_path.write().unwrap();
            *guard = String::from(root_uri.path());
        }
        load_autoload_class_map(&self.parser, &self.state);

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL), // we probably want incremental?
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                        will_save: Some(true),            //idk what this does
                        will_save_wait_until: Some(true), //idk what this does
                    },
                )),
                definition_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        Ok(handle_go_to_definition(
            &params.text_document_position_params.text_document.uri,
            &params.text_document_position_params.position,
            &self.state,
            &self.parser,
        ))
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        handle_did_open(params.text_document, &self.state, &self.parser)
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}
