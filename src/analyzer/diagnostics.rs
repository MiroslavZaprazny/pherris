use std::{process::Command, sync::RwLock};

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range, TextDocumentItem};
use tracing::debug;

use crate::lsp::config::{InitializeOptions, Runtime};

pub struct DiagnosticsError {
    pub message: String,
}

impl DiagnosticsError {
    fn new(message: String) -> Self {
        DiagnosticsError { message }
    }
}

trait DiagnosticCollector: Send + Sync {
    fn collect(&self, document: &TextDocumentItem) -> Result<Vec<Diagnostic>, DiagnosticsError>;
}

struct PhpCliSyntaxDiagnosticCollector {
    php_bin_path: Option<String>,
}

impl DiagnosticCollector for PhpCliSyntaxDiagnosticCollector {
    fn collect(&self, document: &TextDocumentItem) -> Result<Vec<Diagnostic>, DiagnosticsError> {
        let bin = match &self.php_bin_path {
            Some(v) => v,
            None => "php",
        };
        debug!("bin: {}", bin);

        match Command::new(bin)
            .args(["-l", document.uri.to_file_path().unwrap().to_str().unwrap()])
            .output()
        {
            Ok(output) => {
                let mut diagnostics = Vec::new();
                let lines = String::from_utf8_lossy(&output.stderr);

                for line in lines.lines() {
                    if let (Some(start), Some(end), Some(line_num)) = (
                        line.find("syntax error"),
                        line.find(" in "),
                        line.find("on line"),
                    ) {
                        let msg = &line[start + 14..end];
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

impl PhpCliSyntaxDiagnosticCollector {
    fn new(php_bin_path: Option<String>) -> Self {
        PhpCliSyntaxDiagnosticCollector { php_bin_path }
    }
}

struct TsSyntaxDiagnosticCollector {}
impl DiagnosticCollector for TsSyntaxDiagnosticCollector {
    fn collect(&self, _document: &TextDocumentItem) -> Result<Vec<Diagnostic>, DiagnosticsError> {
        Ok(Vec::new())
    }
}

impl TsSyntaxDiagnosticCollector {
    fn new() -> Self {
        TsSyntaxDiagnosticCollector {}
    }
}

struct DockerSyntaxDiagnosticCollector {
    image: String,
}

impl DiagnosticCollector for DockerSyntaxDiagnosticCollector {
    fn collect(&self, document: &TextDocumentItem) -> Result<Vec<Diagnostic>, DiagnosticsError> {
        match Command::new("docker")
            .args([
                "run",
                "--rm",
                "-v",
                &format!(
                    "{}:/file.php",
                    document.uri.to_file_path().unwrap().to_str().unwrap()
                ),
                &self.image,
                "php",
                "-l",
                &format!("/file.php"),
            ])
            .output()
        {
            Ok(output) => {
                debug!("out: {:?}", output);
                let mut diagnostics = Vec::new();
                let lines = String::from_utf8_lossy(&output.stdout);

                for line in lines.lines() {
                    if let (Some(start), Some(end), Some(line_num)) = (
                        line.find("syntax error"),
                        line.find(" in "),
                        line.find("on line"),
                    ) {
                        let msg = &line[start + 14..end];
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
            Err(err) => {
                println!("Output err: {:?}", err);
                Err(DiagnosticsError::new(err.to_string()))
            }
        }
    }
}

impl DockerSyntaxDiagnosticCollector {
    fn new(image: String) -> Self {
        DockerSyntaxDiagnosticCollector { image }
    }
}

struct DockerComposeSyntaxDiagnosticCollector {}
impl DiagnosticCollector for DockerComposeSyntaxDiagnosticCollector {
    fn collect(&self, _document: &TextDocumentItem) -> Result<Vec<Diagnostic>, DiagnosticsError> {
        Ok(Vec::new())
    }
}

impl DockerComposeSyntaxDiagnosticCollector {
    fn new() -> Self {
        DockerComposeSyntaxDiagnosticCollector {}
    }
}

struct SyntaxDiagnosticCollectorFactory {}

impl SyntaxDiagnosticCollectorFactory {
    fn create(options: &RwLock<InitializeOptions>) -> Box<dyn DiagnosticCollector> {
        let options = options.read().unwrap().clone();
        debug!("Options: {:?}", options);

        //eh
        match options.runtime {
            Some(runtime) => match runtime {
                Runtime::PhpCli => {
                    Box::new(PhpCliSyntaxDiagnosticCollector::new(options.php_bin_path))
                }
                Runtime::Docker => {
                    let image = match options.docker_image {
                        Some(v) => v,
                        None => String::from("php:8.4"),
                    };
                    debug!("Image: {:?}", image);

                    Box::new(DockerSyntaxDiagnosticCollector::new(image))
                }
                Runtime::DockerCompose => Box::new(DockerComposeSyntaxDiagnosticCollector::new()),
                Runtime::Ts => Box::new(TsSyntaxDiagnosticCollector::new()),
            },
            None => {
                debug!("defaulting to either php or ts");
                if Command::new("php").arg("-v").output().is_ok() {
                    return Box::new(PhpCliSyntaxDiagnosticCollector::new(options.php_bin_path));
                }

                return Box::new(TsSyntaxDiagnosticCollector {});
            }
        }
    }
}

pub fn collect_diagnostics(
    document: &TextDocumentItem,
    options: &RwLock<InitializeOptions>,
) -> Result<Vec<Diagnostic>, DiagnosticsError> {
    //TODO also add static analysis here
    let syntax_error_collector = SyntaxDiagnosticCollectorFactory::create(options);

    syntax_error_collector.collect(document)
}

#[cfg(test)]
mod tests {
    use std::{io::Write, path::Path};

    use tempfile::TempDir;
    use tower_lsp::lsp_types::{TextDocumentItem, Url};

    use super::{DiagnosticCollector, PhpCliSyntaxDiagnosticCollector};

    #[test]
    fn test_find_parse_errors_using_php_cli() {
        let file_contents = r#"<?php
            $variable = 0
        "#;
        let temp_dir = TempDir::new().expect("to initialize temp dir");
        let path_str = format!("{}/{}", temp_dir.path().to_str().unwrap(), "test.php");
        let temp_dir_path = Path::new(&path_str);
        prepare_php_file(temp_dir_path, file_contents);
        let document = TextDocumentItem::new(
            Url::from_file_path(temp_dir_path).unwrap(),
            String::from(""),
            1,
            file_contents.to_string(),
        );

        let collector = PhpCliSyntaxDiagnosticCollector::new(None);

        match collector.collect(&document) {
            Ok(d) => {
                assert_eq!(1, d.len());
                assert_eq!("unexpected end of file", d.first().unwrap().message);
            }
            Err(e) => panic!(
                "{}",
                format!("Failed to collect diagnostics: {}", e.message)
            ),
        }
    }

    fn prepare_php_file(file_path: &Path, file_contents: &str) {
        let mut file = std::fs::File::create(file_path).expect("to create file");
        file.write_all(file_contents.as_bytes())
            .expect("to write to file");
    }
}
