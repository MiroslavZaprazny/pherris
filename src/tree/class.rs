use tower_lsp::lsp_types::Url;
use tree_sitter::Range;

pub struct ClassDefinition {
    pub uri: Url,
    pub namespace: Option<String>,
}
