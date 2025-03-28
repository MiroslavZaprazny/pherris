use mago_interner::ThreadedInterner;
use mago_lexer::input::Input;
use mago_parser::parse;
use mago_source::{Source, SourceIdentifier};
use mago_span::HasSpan;
use std::sync::RwLock;
use tracing::debug;

use tower_lsp::{
    lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, TextDocumentItem},
    Client,
};

use crate::{analyzer::parser::Parser, lsp::state::State};

pub async fn handle_did_open(
    document: &TextDocumentItem,
    state: &State,
    parser: &RwLock<Parser>,
    client: &Client,
) {
    let interner = ThreadedInterner::new(); // this should probably be in the backend struct
    let source_id = SourceIdentifier::dummy(); // is this right?
                                               // instead of initializing a standolone source we should probably intiliaze a sourcemanager
                                               // somewhere
    let source = Source::standalone(&interner, document.uri.path(), document.text.as_str());
    let input = Input::new(source_id, document.text.as_bytes());
    let (output, error) = parse(&interner, input);
    debug!("Mago parser errror: {:?}", error);
    debug!("Mago parser output soruce identifier: {:?}", output.source);
    debug!(
        "Mago parser output soruce identifier: {:?}",
        output.statements
    );
    debug!("Mago parser error message: {:?}", error);

    if let Some(e) = error {
        let span = e.span();

        let range = Range {
            start: Position {
                line: (source.line_number(span.start.offset)) as u32,
                character: (source.column_number(span.start.offset)) as u32,
            },
            end: Position {
                line: (source.line_number(span.end.offset)) as u32,
                character: (source.column_number(span.end.offset)) as u32,
            },
        };

        let mut diags = vec![];
        diags.push(Diagnostic::new(
            range,
            Some(DiagnosticSeverity::ERROR),
            None,
            None,
            format!("{}", e),
            None,
            None,
        ));

        client
            .publish_diagnostics(document.uri.clone(), diags, None)
            .await;
    }

    // let tree = parser
    //     .write()
    //     .unwrap()
    //     .parse(document.text.clone())
    //     .expect("to parse file");
    // let uri = document.uri.clone();
    // state.ast_map.insert(uri.clone(), tree);
    // state
    //     .document_map
    //     .insert(uri.clone(), document.text.clone());
}
