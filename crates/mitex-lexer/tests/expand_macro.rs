mod common;

use common::*;
use mitex_lexer::MacroEngine;

#[test]
fn base() {
    let engine = MacroEngine::new("hello world", DEFAULT_SPEC.clone());
    let _ = engine;
}
