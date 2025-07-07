use std::sync::RwLock;

use mago_interner::ThreadedInterner;
use mago_lexer::input::Input;
use mago_parser::parse;
use mago_source::{Source, SourceIdentifier};
use mago_span::HasSpan;

use tower_lsp::{
    lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, TextDocumentItem},
    Client,
};

use crate::{analyzer::parser::Parser, lsp::state::State};

pub async fn handle_did_open(
    document: &TextDocumentItem,
    state: &State,
    client: &Client,
    parser: &RwLock<Parser>,
) {
    let interner = ThreadedInterner::new();
    let source_id = SourceIdentifier::dummy();
    let source = Source::standalone(&interner, document.uri.path(), document.text.as_str()); // is this right?
                                                                                             // instead of initializing a standolone source we should probably intiliaze a sourcemanager
                                                                                             // somewhere
    let input = Input::new(source_id, document.text.as_bytes());
    let (program, error) = parse(&interner, input);

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

        let diags = vec![Diagnostic::new(
            range,
            Some(DiagnosticSeverity::ERROR),
            None,
            None,
            e.to_string(),
            None,
            None,
        )];

        client
            .publish_diagnostics(document.uri.clone(), diags, None)
            .await;
    }

    state.document_program.insert(document.uri.clone(), program);
    state
        .document_map
        .insert(document.uri.clone(), document.text.clone());

    //todo remove after we ditch tree sitter for mago parser
    let tree = parser
        .write()
        .unwrap()
        .parse(document.text.clone())
        .expect("to parse file");
    state.ast_map.insert(document.uri.clone(), tree);
}
