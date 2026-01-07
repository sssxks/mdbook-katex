use super::*;
use mdbook_preprocessor::book::Book;
use std::io::Cursor;

fn minimal_ctx_json(mdbook_version: &str) -> serde_json::Value {
    serde_json::json!({
        "root": ".",
        "config": {},
        "renderer": "html",
        "mdbook_version": mdbook_version
    })
}

#[test]
fn test_parse_input_compat_mdbook_05_shape() {
    let input = serde_json::json!([
        minimal_ctx_json("0.5.2"),
        { "items": [] }
    ]);
    let reader = Cursor::new(serde_json::to_vec(&input).unwrap());
    let (_ctx, book) = parse_input_compat(reader).unwrap();
    debug_assert_eq!(book, Book::default());
}

#[test]
fn test_parse_input_compat_mdbook_04_shape_sections_alias() {
    let input = serde_json::json!([
        minimal_ctx_json("0.4.40"),
        { "sections": [] }
    ]);
    let reader = Cursor::new(serde_json::to_vec(&input).unwrap());
    let (_ctx, book) = parse_input_compat(reader).unwrap();
    debug_assert_eq!(book, Book::default());
}
