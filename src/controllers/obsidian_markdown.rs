//! Obsidian markdown extensions applied to the HTML that the `markdown` crate
//! renders: callouts (`> [!note] Title`) and highlights (`==text==`).

/// Turns blockquotes starting with `[!type]` into callouts. `[!type]-` and
/// `[!type]+` make them foldable, collapsed or expanded. Text after the type
/// is the title, which defaults to the type itself.
pub fn render_callouts(html: &str) -> String {
    const OPEN: &str = "<blockquote>";
    let mut result = String::with_capacity(html.len());
    let mut rest = html;

    while let Some(start) = rest.find(OPEN) {
        result.push_str(&rest[..start]);
        let inner_start = start + OPEN.len();
        let Some(inner_len) = matching_blockquote_end(&rest[inner_start..]) else {
            result.push_str(&rest[start..]);
            return result;
        };
        let inner = &rest[inner_start..inner_start + inner_len];
        let end = inner_start + inner_len + "</blockquote>".len();

        match parse_callout(inner) {
            Some(callout) => result.push_str(&callout.to_html()),
            None => {
                result.push_str(OPEN);
                result.push_str(&render_callouts(inner));
                result.push_str("</blockquote>");
            }
        }
        rest = &rest[end..];
    }

    result.push_str(rest);
    result
}

/// Length of a blockquote's content, up to its own closing tag.
fn matching_blockquote_end(html: &str) -> Option<usize> {
    let mut depth = 0;
    let mut offset = 0;

    loop {
        let next_open = html[offset..].find("<blockquote>");
        let next_close = html[offset..].find("</blockquote>")?;
        match next_open {
            Some(open) if open < next_close => {
                depth += 1;
                offset += open + "<blockquote>".len();
            }
            _ if depth == 0 => return Some(offset + next_close),
            _ => {
                depth -= 1;
                offset += next_close + "</blockquote>".len();
            }
        }
    }
}

struct Callout<'a> {
    kind: String,
    fold: Option<bool>,
    title: Option<&'a str>,
    body: String,
}

fn parse_callout(inner: &str) -> Option<Callout<'_>> {
    let first = inner.trim_start().strip_prefix("<p>[!")?;
    let kind_len = first.find(']')?;
    let kind = &first[..kind_len];
    if kind.is_empty() || !kind.chars().all(|c| c.is_alphanumeric() || c == '-') {
        return None;
    }

    let after = &first[kind_len + 1..];
    let (fold, after) = match after.as_bytes().first() {
        Some(b'-') => (Some(false), &after[1..]),
        Some(b'+') => (Some(true), &after[1..]),
        _ => (None, after),
    };

    let paragraph_end = after.find("</p>")?;
    let paragraph = &after[..paragraph_end];
    let (title, first_lines) = paragraph.split_once('\n').unwrap_or((paragraph, ""));
    let mut body = String::new();
    if !first_lines.trim().is_empty() {
        body.push_str(&format!("<p>{first_lines}</p>"));
    }
    body.push_str(&render_callouts(&after[paragraph_end + "</p>".len()..]));

    let title = title.trim();
    Some(Callout {
        kind: kind.to_lowercase(),
        fold,
        title: (!title.is_empty()).then_some(title),
        body: body.trim().to_string(),
    })
}

impl Callout<'_> {
    fn to_html(&self) -> String {
        let title = match self.title {
            Some(title) => title.to_string(),
            None => {
                let mut chars = self.kind.chars();
                chars
                    .next()
                    .map(|c| c.to_uppercase().chain(chars).collect())
                    .unwrap_or_default()
            }
        };
        let content = match self.body.is_empty() {
            true => String::new(),
            false => format!("<div class=\"callout-content\">{}</div>", self.body),
        };

        // `kind` is only alphanumerics and dashes, and `title` is already escaped HTML
        match self.fold {
            None => format!(
                "<div class=\"callout\" data-callout=\"{}\"><div class=\"callout-title\">{title}</div>{content}</div>",
                self.kind
            ),
            Some(open) => format!(
                "<details class=\"callout\" data-callout=\"{}\"{}><summary class=\"callout-title\">{title}</summary>{content}</details>",
                self.kind,
                if open { " open" } else { "" }
            ),
        }
    }
}

/// Wraps `==text==` in `<mark>`. Both markers must be on the same line, and
/// code and the inside of HTML tags are left alone.
pub fn render_highlights(html: &str) -> String {
    let bytes = html.as_bytes();
    let mut markers = Vec::new();
    let mut in_tag = false;
    let mut i = 0;

    while i < bytes.len() {
        let rest = &html[i..];
        if !in_tag && (rest.starts_with("<code") || rest.starts_with("<pre")) {
            let closing = if rest.starts_with("<code") {
                "</code>"
            } else {
                "</pre>"
            };
            i += rest
                .find(closing)
                .map_or(rest.len(), |end| end + closing.len());
            continue;
        }

        match bytes[i] {
            b'<' => in_tag = true,
            b'>' => in_tag = false,
            b'=' if !in_tag
                && bytes.get(i + 1) == Some(&b'=')
                && bytes.get(i + 2) != Some(&b'=')
                && (i == 0 || bytes[i - 1] != b'=') =>
            {
                markers.push(i);
                i += 2;
                continue;
            }
            _ => {}
        }
        i += 1;
    }

    let mut result = String::with_capacity(html.len());
    let mut last = 0;
    let mut pair = markers.windows(2).map(|w| (w[0], w[1])).peekable();
    while let Some((open, close)) = pair.next() {
        let text = &html[open + 2..close];
        let valid = !text.is_empty()
            && !text.contains('\n')
            && !text.starts_with(char::is_whitespace)
            && !text.ends_with(char::is_whitespace);
        if !valid {
            continue;
        }

        result.push_str(&html[last..open]);
        result.push_str("<mark>");
        result.push_str(text);
        result.push_str("</mark>");
        last = close + 2;
        // The closing marker can't open the next highlight
        pair.next();
    }

    result.push_str(&html[last..]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn markdown(md: &str) -> String {
        render_callouts(&markdown::to_html(md))
    }

    #[test]
    fn blockquotes_with_a_type_become_callouts() {
        assert_eq!(
            markdown("> [!warning] Mind the *fog*\n> It never lifts."),
            "<div class=\"callout\" data-callout=\"warning\"><div class=\"callout-title\">Mind the <em>fog</em></div><div class=\"callout-content\"><p>It never lifts.</p></div></div>"
        );
    }

    #[test]
    fn callout_titles_default_to_the_type() {
        let html = markdown("> [!TIP]\n>\n> First.\n>\n> Second.");

        assert!(html.starts_with(
            "<div class=\"callout\" data-callout=\"tip\"><div class=\"callout-title\">Tip</div>"
        ));
        assert!(html.contains("<p>First.</p>\n<p>Second.</p>"));
    }

    #[test]
    fn foldable_callouts_use_details() {
        assert!(
            markdown("> [!faq]- Why?\n> Because.")
                .starts_with("<details class=\"callout\" data-callout=\"faq\"><summary")
        );
        assert!(markdown("> [!faq]+ Why?\n> Because.").contains("data-callout=\"faq\" open>"));
    }

    #[test]
    fn title_only_callouts_have_no_content() {
        assert_eq!(
            markdown("> [!note] Just a title"),
            "<div class=\"callout\" data-callout=\"note\"><div class=\"callout-title\">Just a title</div></div>"
        );
    }

    #[test]
    fn nested_callouts_and_plain_blockquotes() {
        let html = markdown("> [!note] Outer\n> > [!bug] Inner\n> > Crash\n\n> Just a quote");

        assert!(html.contains("data-callout=\"note\""));
        assert!(html.contains("data-callout=\"bug\""));
        assert!(html.contains("<p>Crash</p>"));
        assert!(html.ends_with("<blockquote>\n<p>Just a quote</p>\n</blockquote>"));
    }

    #[test]
    fn invalid_types_stay_blockquotes() {
        assert!(markdown("> [!no way] x").starts_with("<blockquote>"));
        assert!(markdown("> [!] x").starts_with("<blockquote>"));
    }

    #[test]
    fn highlights_wrap_text_between_double_equals() {
        assert_eq!(
            render_highlights("<p>A ==bright **idea**== and ==<em>more</em>==.</p>"),
            "<p>A <mark>bright **idea**</mark> and <mark><em>more</em></mark>.</p>"
        );
    }

    #[test]
    fn highlights_skip_code_attributes_and_broken_pairs() {
        for html in [
            "<p><code>a ==b==</code></p>",
            "<pre><code>==x==\n</code></pre>",
            "<p><a href=\"/x?a==b==\">link</a></p>",
            "<p>== spaced == and a === b ===</p>",
            "<p>==line\nbreak==</p>",
        ] {
            assert_eq!(render_highlights(html), html, "{html}");
        }
    }

    #[test]
    fn unmatched_markers_do_not_shift_the_pairs() {
        assert_eq!(
            render_highlights("<p>a == b ==ok== c</p>"),
            "<p>a == b <mark>ok</mark> c</p>"
        );
    }
}
