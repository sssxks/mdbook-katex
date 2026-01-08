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

/// Escape `_` characters in KaTeX-rendered HTML text nodes.
///
/// KaTeX renders `\_` (escaped underscore in TeX) as a literal `_` character. When we inject the
/// resulting HTML back into markdown, `pulldown-cmark` can interpret that `_` as an emphasis
/// delimiter, producing broken HTML (e.g. inserting `<em>` inside KaTeX output).
///
/// We only escape underscores that appear outside of tags (i.e. in HTML text nodes) to avoid
/// surprising changes to attribute values. If no escaping is needed, this returns a borrowed view.
fn escape_underscores_for_markdown(rendered_html: &str) -> Cow<'_, str> {
    let mut in_tag = false;
    let mut cursor = 0;
    let mut output: Option<String> = None;

    for (index, ch) in rendered_html.char_indices() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            '_' if !in_tag => {
                let out = output.get_or_insert_with(|| String::with_capacity(rendered_html.len() + 16));
                out.push_str(&rendered_html[cursor..index]);
                out.push_str("&#95;");
                cursor = index + ch.len_utf8();
            }
            _ => {}
        }
    }

    match output {
        Some(mut out) => {
            out.push_str(&rendered_html[cursor..]);
            Cow::Owned(out)
        }
        None => Cow::Borrowed(rendered_html),
    }
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
            // Escape underscores to prevent markdown parser from interpreting them as emphasis
            let rendered = escape_underscores_for_markdown(&rendered);
            if extra_opts.include_src {
                // Wrap around with `data.katex-src` tag.
                rendered_content.push_str(r#"<data class="katex-src" value=""#);
                rendered_content
                    .push_str(&item_dequoted.replace('"', r#"\""#).replace('\n', r"&#10;"));
                rendered_content.push_str(r#"">"#);
                rendered_content.push_str(rendered.as_ref());
                rendered_content.push_str(r"</data>");
            } else {
                rendered_content.push_str(rendered.as_ref());
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
