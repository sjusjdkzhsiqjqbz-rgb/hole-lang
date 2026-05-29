use crate::ast::{Span, Type};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct HoleInfo {
    pub line: usize,
    pub col: usize,
    pub expected_type: String,
    pub context: String,
}

#[derive(Debug, Serialize)]
pub struct HoleReport {
    pub holes: Vec<HoleInfo>,
}

impl HoleReport {
    pub fn empty() -> Self {
        HoleReport { holes: vec![] }
    }
}

pub fn analyze_holes(holes: &[(Span, Type)], source: &str) -> HoleReport {
    let lines: Vec<&str> = source.lines().collect();
    let report: Vec<HoleInfo> = holes
        .iter()
        .map(|(span, ty)| {
            let context = if span.line > 0 && span.line <= lines.len() {
                let line = lines[span.line - 1];
                let trimmed = line.trim();
                if trimmed.len() > 80 {
                    format!("{}...", &trimmed[..80.min(trimmed.len())])
                } else {
                    trimmed.to_string()
                }
            } else {
                String::new()
            };
            HoleInfo {
                line: span.line,
                col: span.col,
                expected_type: format!("{}", ty),
                context,
            }
        })
        .collect();

    HoleReport { holes: report }
}
