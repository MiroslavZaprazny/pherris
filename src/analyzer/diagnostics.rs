use std::process::Command;
use tracing::debug;

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, TextDocumentItem};

enum DiagnosticRuntime {
    LOCAL,
    TS,
}

impl DiagnosticRuntime {
    fn detect() -> Self {
        if Command::new("php").arg("-v").output().is_ok() {
            return Self::LOCAL;
        }

        Self::TS
    }
}

pub struct DiagnosticsError {
    pub message: String,
}

impl DiagnosticsError {
    fn new(message: String) -> Self {
        DiagnosticsError { message }
    }
}

pub trait DiagnosticCollector: Send + Sync {
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
                let mut diagnostics = Vec::new();
                let lines = String::from_utf8_lossy(&output.stderr);

                for line in lines.lines() {
                    debug!("line: {:?}", line);
                    if !line.contains("PHP Parse error:  syntax error,") {
                        continue;
                    }

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
                }

                Ok(diagnostics)
            }
            Err(err) => Err(DiagnosticsError::new(err.to_string())),
        }
    }
}

struct TsDiagnosticCollector {}
impl DiagnosticCollector for TsDiagnosticCollector {
    fn collect(&self, _document: &TextDocumentItem) -> Result<Vec<Diagnostic>, DiagnosticsError> {
        Ok(Vec::new())
    }
}

impl TsDiagnosticCollector {
    fn new() -> Self {
        TsDiagnosticCollector {}
    }
}

impl PhpCliDiagnosticCollector {
    fn new() -> Self {
        PhpCliDiagnosticCollector {}
    }
}

pub struct DiagnosticCollectorFactory {}

impl DiagnosticCollectorFactory {
    pub fn create() -> Box<dyn DiagnosticCollector> {
        match DiagnosticRuntime::detect() {
            // DiagnosticRuntime::TS => Box::new(TsDiagnosticCollector::new()),
            DiagnosticRuntime::LOCAL => Box::new(PhpCliDiagnosticCollector::new()),
            DiagnosticRuntime::TS => Box::new(TsDiagnosticCollector::new()),
        }
    }
}
