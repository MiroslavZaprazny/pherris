use streaming_iterator::StreamingIterator;
use tower_lsp::lsp_types::{Location, Range, Url};
use tracing::debug;
use tree_sitter::{Query, QueryCursor, Tree};

use crate::analyzer::utils::get_position_from_point;

pub fn get_variable_locations_for_query(
    var_name: &str,
    query: &Query,
    tree: &Tree,
    document: &str,
    uri: &Url,
) -> Vec<Location> {
    let mut out = Vec::new();
    let mut cursor = QueryCursor::new();

    let mut matches = cursor.matches(&query, tree.root_node(), document.as_bytes());

    while let Some(match_) = matches.next() {
        debug!("Match {:?}", match_);
        for capture in match_.captures {
            let declare_var_name = capture
                .node
                .utf8_text(document.as_bytes())
                .expect("a text")
                .trim_start_matches("$");

            if declare_var_name == var_name {
                let range = Range::new(
                    get_position_from_point(&capture.node.start_position()),
                    get_position_from_point(&capture.node.end_position()),
                );

                out.push(Location::new(uri.clone(), range))
            }
        }
    }

    out
}
