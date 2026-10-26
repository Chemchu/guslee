/// An Obsidian-style `#tag` written in a post's text.
#[derive(Debug, PartialEq, Eq)]
pub struct InlineTag<'a> {
    /// Byte range of the whole `#tag`, including the `#`.
    pub start: usize,
    pub end: usize,
    /// The tag without the `#`. Nested tags keep their slashes: `gaming/horror`.
    pub name: &'a str,
}

fn is_tag_char(c: char) -> bool {
    c.is_alphanumeric() || matches!(c, '_' | '-' | '/')
}

/// `#2F3E46` and friends are colours, not tags.
fn looks_like_hex_colour(name: &str) -> bool {
    matches!(name.len(), 3 | 4 | 6 | 8)
        && name.chars().all(|c| c.is_ascii_hexdigit())
        && name.chars().any(|c| c.is_ascii_digit())
}

/// Finds `#tags` in plain text (no markup or code). Like Obsidian, a tag starts
/// at the beginning of the text or after whitespace and needs a non-digit.
pub fn find_inline_tags(text: &str) -> Vec<InlineTag<'_>> {
    let mut tags = Vec::new();
    let mut previous: Option<char> = None;

    for (start, c) in text.char_indices() {
        let at_boundary = previous.is_none_or(char::is_whitespace);
        previous = Some(c);
        if c != '#' || !at_boundary {
            continue;
        }

        let name_start = start + 1;
        let name_len = text[name_start..]
            .find(|c: char| !is_tag_char(c))
            .unwrap_or(text.len() - name_start);
        let name = text[name_start..name_start + name_len].trim_end_matches(['/', '-']);

        let has_non_digit = name.chars().any(|c| !c.is_ascii_digit());
        if name.starts_with('/') || !has_non_digit || looks_like_hex_colour(name) {
            continue;
        }
        tags.push(InlineTag {
            start,
            end: name_start + name.len(),
            name,
        });
    }

    tags
}

/// The inline `#tags` of a markdown post, in order and without repeats.
/// Frontmatter and code are skipped.
pub fn markdown_inline_tags(md: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    let mut in_fence = false;
    let mut lines = md.lines().peekable();

    if lines.peek().is_some_and(|line| line.trim_end() == "---") {
        lines.next();
        for line in lines.by_ref() {
            if line.trim_end() == "---" {
                break;
            }
        }
    }

    for line in lines {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }

        // Every other backtick-separated piece is an inline code span
        for text in line.split('`').step_by(2) {
            for tag in find_inline_tags(text) {
                if !found.iter().any(|f| f.eq_ignore_ascii_case(tag.name)) {
                    found.push(tag.name.to_string());
                }
            }
        }
    }

    found
}

/// Whether a post tagged `tag` belongs on the page of `wanted`. Nested tags
/// count for their parents: `gaming/horror` is listed under `gaming`.
pub fn tag_matches(tag: &str, wanted: &str) -> bool {
    let (tag, wanted) = (tag.to_lowercase(), wanted.to_lowercase());
    tag == wanted || tag.starts_with(&format!("{wanted}/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(text: &str) -> Vec<&str> {
        find_inline_tags(text).into_iter().map(|t| t.name).collect()
    }

    #[test]
    fn finds_tags_after_whitespace_or_at_the_start() {
        assert_eq!(
            names("#intro and #gaming/horror, then #año_2025."),
            vec!["intro", "gaming/horror", "año_2025"]
        );
    }

    #[test]
    fn ignores_hashes_inside_words_urls_and_entities() {
        assert!(names("issue#12 site.com/#about it&#39;s C# ## #").is_empty());
    }

    #[test]
    fn needs_a_non_digit_and_skips_colours() {
        assert_eq!(
            names("#1 #2025 #2F3E46 #fff #add #4ever"),
            vec!["fff", "add", "4ever"]
        );
    }

    #[test]
    fn trailing_separators_are_not_part_of_the_tag() {
        let tags = find_inline_tags("so #meta- #nested/ end");

        assert_eq!(tags[0].name, "meta");
        assert_eq!(
            &"so #meta- #nested/ end"[tags[0].start..tags[0].end],
            "#meta"
        );
        assert_eq!(tags[1].name, "nested");
    }

    #[test]
    fn markdown_tags_skip_frontmatter_and_code() {
        let md = "---\ntitle: \"#nope\"\n---\n# Heading #real\n\nText #Real `#code` #inline\n\n```\n#fenced\n```\n#after";

        assert_eq!(markdown_inline_tags(md), vec!["real", "inline", "after"]);
    }

    #[test]
    fn nested_tags_belong_to_their_parents() {
        assert!(tag_matches("Gaming", "gaming"));
        assert!(tag_matches("gaming/horror", "gaming"));
        assert!(tag_matches("gaming/horror", "Gaming/Horror"));
        assert!(!tag_matches("gaming", "gaming/horror"));
        assert!(!tag_matches("gamingx", "gaming"));
    }
}
