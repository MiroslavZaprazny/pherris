use mago_ast::{ClassLikeMember, Node, NodeKind};
use mago_interner::ThreadedInterner;
use mago_lexer::input::Input;
use mago_names::Names;
use mago_parser::parse;
use mago_source::{Source, SourceIdentifier};
use mago_span::{HasPosition, HasSpan};
use std::sync::RwLock;
use tracing::debug;
use tracing_subscriber::field::debug;

use tower_lsp::{
    lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, TextDocumentItem},
    Client,
};

use crate::{analyzer::parser::Parser, lsp::state::State};

fn test(node: &Node, contents: &str, source: &Source, current: &Position) {
    debug!("Kind: {:?}", node.kind());

    if let Node::ClassLikeMember(n) = node {
        debug!("class like node: {:?}", n);
        let text = &contents[node.start_position().offset()..node.end_position().offset()];
        let range = Range {
            start: Position {
                line: (source.line_number(node.start_position().offset())) as u32,
                character: (source.column_number(node.start_position().offset())) as u32,
            },
            end: Position {
                line: (source.line_number(node.end_position().offset())) as u32,
                character: (source.column_number(node.end_position().offset())) as u32,
            },
        };
        if range.start.line <= current.line && range.end.line >= current.line {
            debug!("SOM TU HALO");
        }

        // let name = names.get(&node.position());
        debug!("text: {:?}", text);
        debug!("range: {:?}", range);
    }

    for node in node.children() {
        test(&node, contents, source, current);
    }
}

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
    let (program, error) = parse(&interner, input);
    let root_node = Node::Program(&program);
    // let names = Names::resolve(&interner, &program);

    //just fucking around
    //move to handle_go_to_definition
    let current_position = Position::new(10, 42);
    for child in root_node.children() {
        test(&child, document.text.as_str(), &source, &current_position);
    }

    // debug!("{:?}", names);

    // let test = Node::Program(&program);
    // let node_children = test.children();
    //
    // let mut children = Vec::with_capacity(node_children.len());
    // for child in node_children {
    //     children.push(child);
    // }
    // debug!("Nodes: {:?}", children);

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
