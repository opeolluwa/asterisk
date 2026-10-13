//! The one place the reply-HTML allowlist is defined.
//!
//! Reply bodies arrive as HTML from the console's rich text editor and are
//! sanitized twice by design:
//!
//! 1. On write, in `ComplaintService::reply_to_complaint`, so an unsanitized
//!    string is never persisted.
//! 2. On render, in the complaint reply dispatch, because `complaint_reply.html`
//!    is the first template here to interpolate with `| safe` — the alternative
//!    is either broken formatting or an injection vector into customer email.
//!
//! One shared policy across both passes is the point. The console also runs
//! DOMPurify before `v-html`, but that is a client convenience any HTTP client
//! can bypass, not a boundary.

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use ammonia::Builder;

/// Tags the editor's `StarterKit` + `BubbleMenu` toolbar can emit.
///
/// This *replaces* ammonia's default set rather than extending it, so it has to
/// be a deliberate superset of the editor's schema: paragraph, all six heading
/// levels, both list kinds, blockquote, preformatted and inline code, thematic
/// break, hard break, the inline marks, and `a` for links. Narrowing this list
/// is the intended way to tighten what a reply may contain — a tag missing here
/// is silently stripped, not rejected, so the console loses formatting without
/// any error surfacing.
const ALLOWED_TAGS: &[&str] = &[
    "a",
    "blockquote",
    "br",
    "code",
    "del",
    "em",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "hr",
    "i",
    "li",
    "ol",
    "p",
    "pre",
    "s",
    "span",
    "strong",
    "sub",
    "sup",
    "u",
    "ul",
];

/// Per-tag attributes worth keeping. Anything not listed is dropped.
const ALLOWED_ATTRIBUTES: &[(&str, &[&str])] = &[
    ("a", &["href", "title"]),
    ("span", &["style"]),
    ("p", &["style"]),
];

/// Builds the configured sanitizer.
///
/// Constructed per call rather than cached in a `LazyLock`: `ammonia::Builder`
/// is not `Sync`, and the build is cheap relative to the database round trips
/// that surround it.
fn builder() -> Builder<'static> {
    let tags: HashSet<&'static str> = ALLOWED_TAGS.iter().copied().collect();
    let tag_attributes: HashMap<&'static str, HashSet<&'static str>> = ALLOWED_ATTRIBUTES
        .iter()
        .map(|(tag, attributes)| (*tag, attributes.iter().copied().collect()))
        .collect();

    let mut builder = Builder::default();

    builder
        .tags(tags)
        .tag_attributes(tag_attributes)
        .attribute_filter(|_element, attribute, value| {
            // Ammonia's default `url_schemes` already excludes `javascript:`,
            // but `href` is the one attribute in the allowlist that can carry
            // script, so it is re-checked rather than inherited.
            if attribute == "href" {
                let lowered = value.trim_start().to_ascii_lowercase();
                if lowered.starts_with("javascript:") || lowered.starts_with("vbscript:") {
                    return None;
                }
            }

            // Inline styles are limited to colour/typography. `position` would
            // let a reply overlay or displace the template chrome, and `url()`
            // re-opens remote content loading.
            if attribute == "style" {
                let lowered = value.to_ascii_lowercase();
                if lowered.contains("position")
                    || lowered.contains("expression")
                    || lowered.contains("url(")
                    || lowered.contains("behavior")
                    || lowered.contains("@import")
                {
                    return None;
                }
            }

            Some(Cow::Borrowed(value))
        })
        .link_rel(Some("noopener noreferrer"))
        .strip_comments(true);

    builder
}

/// Strips everything outside the reply allowlist from `html`.
///
/// Returns cleaned, well-formed HTML. Callers must run this on both the write
/// and the render path; see the module docs for why.
pub fn clean_reply_html(html: &str) -> String {
    builder().clean(html).to_string()
}

#[cfg(test)]
mod tests {
    use super::clean_reply_html;

    #[test]
    fn strips_script_tags_and_their_contents() {
        let cleaned = clean_reply_html("<p>Hello</p><script>alert(1)</script>");

        assert!(cleaned.contains("Hello"));
        assert!(!cleaned.contains("script"));
    }

    #[test]
    fn strips_event_handler_attributes() {
        let cleaned = clean_reply_html(r#"<img src="x" onerror="alert(1)">"#);

        assert!(!cleaned.contains("onerror"));
    }

    #[test]
    fn drops_image_tags_entirely() {
        // Not in the allowlist, so the tag goes — which also removes the
        // attribute that made images a convenient `onerror` carrier.
        let cleaned = clean_reply_html(r#"<p>a</p><img src=x onerror=alert(1)>"#);

        assert!(cleaned.contains("a"));
        assert!(!cleaned.contains("img"));
    }

    #[test]
    fn rejects_javascript_hrefs() {
        let cleaned = clean_reply_html(r#"<a href="javascript:alert(1)">x</a>"#);

        assert!(!cleaned.to_lowercase().contains("javascript:"));
    }

    #[test]
    fn keeps_editor_formatting() {
        let cleaned = clean_reply_html(
            "<p><strong>bold</strong> and <em>italic</em></p><ul><li>one</li></ul><blockquote>q</blockquote>",
        );

        assert!(cleaned.contains("<strong>bold</strong>"));
        assert!(cleaned.contains("<em>italic</em>"));
        assert!(cleaned.contains("<ul>"));
        assert!(cleaned.contains("<li>one</li>"));
        assert!(cleaned.contains("<blockquote>"));
    }

    #[test]
    fn allows_safe_hrefs() {
        let cleaned = clean_reply_html(r#"<a href="https://example.com">link</a>"#);

        assert!(cleaned.contains("https://example.com"));
    }

    #[test]
    fn rejects_positioning_styles() {
        let cleaned = clean_reply_html(r#"<p style="position:fixed;top:0">x</p>"#);

        assert!(!cleaned.contains("position"));
    }
}
