use std::process::Command;
use tracing::debug;

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, TextDocumentItem};

pub struct DiagnosticsError {
    pub message: String,
}

impl DiagnosticsError {
    fn new(message: String) -> Self {
        DiagnosticsError { message }
    }
}

pub trait DiagnosticCollector {
    fn collect(&self, document: &TextDocumentItem) -> Result<Vec<Diagnostic>, DiagnosticsError>;
}

struct PhpCliDiagnosticCollector {}

impl DiagnosticCollector for PhpCliDiagnosticCollector {
    fn collect(&self, document: &TextDocumentItem) -> Result<Vec<Diagnostic>, DiagnosticsError> {
        debug!("Collecting diagnostics");

        match Command::new("php")
            .args(["-l", document.uri.to_file_path().unwrap().to_str().unwrap()])
            .output()
        {
            Ok(output) => {
                debug!("output: {:?}", output);

                let mut diagnostics = Vec::new();
                let lines = String::from_utf8_lossy(&output.stderr);

                for line in lines.lines() {
                    debug!("line: {:?}", line);
                    if !line.contains("PHP Parse error:  syntax error,") {
                        continue;
                    }

                    debug!("has parse error: {:?}", true);
                    debug!("1: {:?}", line.find("syntax error"));
                    debug!("2: {:?}", line.find("in"));
                    debug!("3: {:?}", line.find("on line"));

                    if let (Some(start), Some(end), Some(line_num)) = (
                        line.find("syntax error"),
                        line.find(" in "),
                        line.find("on line"),
                    ) {
                        let msg = &line[start + 14..end];
                        debug!("line num: {:?}", &line[line_num + 8..]);
                        let line_num = &line[line_num + 8..]
                            .parse::<u32>()
                            .expect("to parse line number")
                            - 1;

                        let diagnostic = Diagnostic::new(
                            Range::new(Position::new(line_num, 0), Position::new(line_num, 0)),
                            Some(DiagnosticSeverity::ERROR),
                            None,
                            None,
                            msg.to_string(),
                            None,
                            None,
                        );

                        diagnostics.push(diagnostic);
                    }

                    debug!("Unable to parse error message for line: {}", line)
                }

                Ok(diagnostics)
            }
            Err(err) => Err(DiagnosticsError::new(err.to_string())),
        }
    }
}

impl PhpCliDiagnosticCollector {
    fn new() -> PhpCliDiagnosticCollector {
        PhpCliDiagnosticCollector {}
    }
}

//TODO determine the implementation based on some config?
pub struct DiagnosticCollectorFactory {}

impl DiagnosticCollectorFactory {
    pub fn create() -> impl DiagnosticCollector {
        PhpCliDiagnosticCollector::new()
    }
}
