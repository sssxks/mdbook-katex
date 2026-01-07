//! Render KaTeX math block to HTML
use katex::{Error, Opts};

use super::*;

pub use {cfg::*, preprocess::*};

mod cfg;
mod preprocess;

fn normalize_tex_for_katex(tex: &str) -> Cow<'_, str> {
    // `katex` (via embedded JS engines) currently produces broken HTML for `\backslash`,
    // emitting literal `</span>` text in the output which then triggers pulldown-cmark
    // "unclosed HTML tag" warnings in mdBook.
    //
    // `\setminus` is the same glyph for the common set/quotient notation use case and
    // avoids the broken output.
    if tex.contains(r"\backslash") {
        Cow::Owned(tex.replace(r"\backslash", r"\setminus"))
    } else {
        Cow::Borrowed(tex)
    }
}

fn strip_markdown_blockquote_prefix(item: &str) -> Cow<'_, str> {
    let first_non_empty_line_is_blockquote = item
        .lines()
        .find(|line| !line.trim().is_empty())
        .is_some_and(|line| line.trim_start_matches([' ', '\t']).starts_with('>'));
    if !first_non_empty_line_is_blockquote {
        return Cow::Borrowed(item);
    }

    let mut stripped = String::with_capacity(item.len());
    for chunk in item.split_inclusive('\n') {
        let (line, suffix) = chunk
            .strip_suffix('\n')
            .map(|line| (line, "\n"))
            .unwrap_or((chunk, ""));

        let mut index = 0;
        let bytes = line.as_bytes();
        while index < bytes.len() && (bytes[index] == b' ' || bytes[index] == b'\t') {
            index += 1;
        }
        if index < bytes.len() && bytes[index] == b'>' {
            index += 1;
            if index < bytes.len() && (bytes[index] == b' ' || bytes[index] == b'\t') {
                index += 1;
            }
            stripped.push_str(&line[index..]);
        } else {
            stripped.push_str(line);
        }
        stripped.push_str(suffix);
    }
    Cow::Owned(stripped)
}

/// Render a math block `item` into HTML following `opts`.
/// Wrap result in `<data>` tag if `extra_opts.include_src`.
#[instrument(skip(opts, extra_opts, display))]
pub fn render(item: &str, opts: Opts, extra_opts: ExtraOpts, display: bool) -> String {
    let mut rendered_content = String::new();
    let item_dequoted = strip_markdown_blockquote_prefix(item);
    let normalized_tex = normalize_tex_for_katex(item_dequoted.as_ref());

    // try to render equation
    match katex::render_with_opts(normalized_tex.as_ref(), opts) {
        Ok(rendered) => {
            let rendered = rendered.replace('\n', " ");
            if extra_opts.include_src {
                // Wrap around with `data.katex-src` tag.
                rendered_content.push_str(r#"<data class="katex-src" value=""#);
                rendered_content
                    .push_str(&item_dequoted.replace('"', r#"\""#).replace('\n', r"&#10;"));
                rendered_content.push_str(r#"">"#);
                rendered_content.push_str(&rendered);
                rendered_content.push_str(r"</data>");
            } else {
                rendered_content.push_str(&rendered);
            }
        }
        // if rendering fails, keep the unrendered equation
        Err(why) => {
            match why {
                Error::JsExecError(why) => {
                    warn!("Rendering failed, keeping the original content: {why}")
                }
                _ => error!(
                    ?why,
                    "Unexpected rendering failure, keeping the original content."
                ),
            }
            let delimiter = match display {
                true => &extra_opts.block_delimiter,
                false => &extra_opts.inline_delimiter,
            };
            rendered_content.push_str(&delimiter.left);
            rendered_content.push_str(item);
            rendered_content.push_str(&delimiter.right);
        }
    }

    rendered_content
}
