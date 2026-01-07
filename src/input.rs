//! Compatibility helpers for parsing mdBook's JSON input.
//!
//! mdBook preprocessors receive JSON on stdin, representing `(PreprocessorContext, Book)`.
//! mdBook 0.5 renamed `Book.sections` to `Book.items`, which can break preprocessors built
//! against 0.5 when they are invoked by mdBook 0.4.

use mdbook_preprocessor::{book::Book, errors::Error, errors::Result, PreprocessorContext};
use serde_json::Value;
use std::io::Read;

fn normalize_book_value(book: &mut Value) {
    let Value::Object(map) = book else {
        return;
    };

    if map.contains_key("items") {
        return;
    }

    if let Some(sections) = map.remove("sections") {
        map.insert("items".to_string(), sections);
    }
}

/// Parse preprocessor input from `reader`, supporting both mdBook 0.4 and 0.5 JSON formats.
pub fn parse_input_compat<R: Read>(reader: R) -> Result<(PreprocessorContext, Book)> {
    let value: Value = serde_json::from_reader(reader)
        .map_err(|err| Error::msg(format!("Unable to parse the input: {err}")))?;

    let Value::Array(mut parts) = value else {
        return Err(Error::msg(
            "Unable to parse the input: expected an array [context, book]",
        ));
    };

    if parts.len() != 2 {
        return Err(Error::msg(format!(
            "Unable to parse the input: expected 2 elements, got {}",
            parts.len()
        )));
    }

    let ctx_value = parts.remove(0);
    let mut book_value = parts.remove(0);
    normalize_book_value(&mut book_value);

    let ctx: PreprocessorContext = serde_json::from_value(ctx_value)
        .map_err(|err| Error::msg(format!("Unable to parse the input: {err}")))?;
    let book: Book = serde_json::from_value(book_value)
        .map_err(|err| Error::msg(format!("Unable to parse the input: {err}")))?;

    Ok((ctx, book))
}
