use dashmap::DashMap;
use streaming_iterator::StreamingIterator;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use tracing::debug;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::fmt::Subscriber;
use tree_sitter::{Node, Point, Query, QueryCursor, Tree};

#[derive(Debug)]
struct Backend {
    client: Client,
    ast_map: DashMap<Url, Tree>,
    document_map: DashMap<Url, String>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
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

        let tree = self
            .ast_map
            .get(&params.text_document_position_params.text_document.uri)
            .expect("to get the tree");

        let document = self
            .document_map
            .get(&params.text_document_position_params.text_document.uri)
            .expect("to get the document");

        let current_point = get_point_from_position(&params.text_document_position_params.position);
        let current_node = get_node_for_point(&tree, current_point).expect("to get node");

        debug!("Current node {:?}", current_node);

        let var_query_source = r#"
            (name) @variable
        "#;

        let var_query = Query::new(&tree_sitter_php::LANGUAGE_PHP.into(), var_query_source)
            .expect("to create query");
        let mut var_cursor = QueryCursor::new();

        let declare_query_source = r#"
            (assignment_expression
                left: (variable_name) @declaration)
        "#;
        let declare_query = Query::new(&tree_sitter_php::LANGUAGE_PHP.into(), declare_query_source)
            .expect("to create query");
        let mut declare_cursor = QueryCursor::new();

        //this is messy af
        let mut var_matches = var_cursor.matches(&var_query, current_node, document.as_bytes());
        while let Some(var_match) = var_matches.next() {
            debug!("Matches {:?}", var_match);
            for var_capture in var_match.captures.iter() {
                let var_name = var_capture.node.utf8_text(document.as_bytes()).unwrap();
                debug!("Variable name: {:?}", var_name);
                let mut declare_matches =
                    declare_cursor.matches(&declare_query, tree.root_node(), document.as_bytes());
                while let Some(declare_match) = declare_matches.next() {
                    for declare_capture in declare_match.captures.iter() {
                        let declare_var_name = declare_capture
                            .node
                            .utf8_text(document.as_bytes())
                            .unwrap()
                            .trim_start_matches("$");
                        let declare_node = declare_capture.node;
                        debug!("declare capture: {:?}", declare_node);
                        debug!("declare variable name: {:?}", declare_var_name);

                        if declare_var_name == var_name {
                            let range = Range::new(
                                get_position_from_point(&declare_node.start_position()),
                                get_position_from_point(&declare_node.end_position()),
                            );
                            let location = Location::new(
                                params.text_document_position_params.text_document.uri,
                                range,
                            );
                            let response = GotoDefinitionResponse::Scalar(location);

                            // it looks like we open the file again and then move to the location,
                            // idk why, we should just move up to the declaration
                            return Ok(Some(response));
                        }
                    }
                }
            }
        }

        print_node(current_node, 0);

        // we should probably return some error or smth
        let range = Range::new(
            params.text_document_position_params.position,
            params.text_document_position_params.position,
        );
        let location = Location::new(
            params.text_document_position_params.text_document.uri,
            range,
        );
        let response = GotoDefinitionResponse::Scalar(location);

        Ok(Some(response))
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        debug!("Got a textDocument/didOpen notification");
        if self.ast_map.contains_key(&params.text_document.uri) {
            return;
        }

        let lang = tree_sitter_php::LANGUAGE_PHP;
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&lang.into()).expect("to set lang");

        //should we panic?
        let tree = parser
            .parse(params.text_document.text.clone(), None)
            .expect("to parse file");
        print_tree(&tree);

        self.ast_map.insert(params.text_document.uri.clone(), tree);
        self.document_map
            .insert(params.text_document.uri, params.text_document.text);
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

fn get_point_from_position(position: &Position) -> Point {
    Point {
        row: position.line as usize,
        column: position.character as usize,
    }
}

fn get_position_from_point(point: &Point) -> Position {
    Position {
        line: point.row as u32,
        character: point.column as u32,
    }
}

fn get_node_for_point(tree: &Tree, point: Point) -> Option<Node> {
    return tree.root_node().descendant_for_point_range(point, point);
}

fn print_tree(tree: &Tree) {
    let root_node = tree.root_node();

    print_node(root_node, 0);
}

fn print_node(node: Node, depth: usize) {
    let indent = "  ".repeat(depth);

    debug!(
        "{}{} [{}, {}] - [{}, {}]",
        indent,
        node.kind(),
        node.start_position().row,
        node.start_position().column,
        node.end_position().row,
        node.end_position().column
    );

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        print_node(child, depth + 1);
    }
}

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
    let (service, socket) = LspService::new(|client| Backend {
        client,
        ast_map: DashMap::default(),
        document_map: DashMap::default(),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
