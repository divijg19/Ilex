use std::env;

use super::parsers::parse_terminal_info;
use super::{DetectionError, Detector, SystemSnapshot};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalDetector {
    term_program: Option<String>,
    term: Option<String>,
    color_term: Option<String>,
}

impl Default for TerminalDetector {
    fn default() -> Self {
        Self {
            term_program: env::var("TERM_PROGRAM")
                .ok()
                .filter(|value| !value.trim().is_empty()),
            term: env::var("TERM")
                .ok()
                .filter(|value| !value.trim().is_empty()),
            color_term: env::var("COLORTERM")
                .ok()
                .filter(|value| !value.trim().is_empty()),
        }
    }
}

impl TerminalDetector {
    pub fn new(
        term_program: Option<String>,
        term: Option<String>,
        color_term: Option<String>,
    ) -> Self {
        Self {
            term_program,
            term,
            color_term,
        }
    }
}

impl Detector for TerminalDetector {
    fn key(&self) -> &'static str {
        "terminal"
    }

    fn detect(&self, snapshot: &mut SystemSnapshot) -> Result<(), DetectionError> {
        snapshot.terminal = Some(parse_terminal_info(
            self.term_program.as_deref(),
            self.term.as_deref(),
            self.color_term.as_deref(),
        ));
        Ok(())
    }
}
