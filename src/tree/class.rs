use tower_lsp::lsp_types::{Range, Url};

pub struct ClassDefinition {
    pub uri: Url,
    // pub namespace: Option<String>,
    pub range: Range,
}
