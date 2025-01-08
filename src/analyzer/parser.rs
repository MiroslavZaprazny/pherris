use tree_sitter::{LanguageError, Parser as TSParser, Tree};
use tree_sitter_php::LANGUAGE_PHP;

pub struct Parser {
    inner: TSParser,
}

impl Parser {
    pub fn new() -> Result<Self, LanguageError> {
        let mut parser = TSParser::new();
        parser.set_language(&LANGUAGE_PHP.into())?;

        Ok(Self { inner: parser })
    }

    pub fn parse(&mut self, text: impl AsRef<[u8]>) -> Option<Tree> {
        self.inner.parse(text, None)
    }
}
