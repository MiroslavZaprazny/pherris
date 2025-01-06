use tower_lsp::lsp_types::{Location, Url};
use tree_sitter::{Node, Tree};

pub trait Locator {
    fn find(
        &self,
        current_node: &Node,
        document: &str,
        tree: &Tree,
        current_uri: &Url,
    ) -> Option<Location>;
}

