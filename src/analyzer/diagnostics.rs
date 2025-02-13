use tower_lsp::lsp_types::Diagnostic;
use tree_sitter::Tree;

pub fn collect_diagnostics(tree: &Tree) -> Vec<Diagnostic> {
    Vec::new()
}
