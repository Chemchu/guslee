use std::collections::HashMap;

/// A link to a post in the garden, relative to the garden root.
#[derive(Debug, PartialEq, Eq)]
pub struct LinkTarget {
    /// Garden-relative path without the `.md` extension.
    pub path: String,
    /// Anchor inside the post, without the leading `#`.
    pub fragment: Option<String>,
}

impl LinkTarget {
    pub fn file_path(&self) -> String {
        format!("{}.md", self.path)
    }

    pub fn url(&self) -> String {
        match &self.fragment {
            Some(fragment) => format!("/posts/{}#{}", self.path, fragment),
            None => format!("/posts/{}", self.path),
        }
    }
}

/// Resolves the `href` of a markdown link. External, absolute and anchor-only
/// links are not posts and yield `None`.
pub fn markdown_link_target(href: &str) -> Option<LinkTarget> {
    let has_scheme = href
        .split_once(':')
        .is_some_and(|(scheme, _)| !scheme.contains('/'));
    if href.is_empty() || href.starts_with(['/', '#', '?']) || has_scheme {
        return None;
    }

    let (path, fragment) = match href.split_once('#') {
        Some((path, fragment)) => (path, Some(fragment.to_string())),
        None => (href, None),
    };
    let path = path.trim_start_matches("./");
    let path = path.strip_suffix(".md").unwrap_or(path);

    Some(LinkTarget {
        path: path.to_string(),
        fragment,
    })
}

/// An Obsidian `[[target#heading|alias]]` link.
#[derive(Debug, PartialEq, Eq)]
pub struct WikiLink<'a> {
    /// Note name or path as written. Empty for `[[#heading]]` links to the same note.
    pub target: &'a str,
    pub heading: Option<&'a str>,
    pub alias: Option<&'a str>,
}

impl WikiLink<'_> {
    pub fn parse(inner: &str) -> WikiLink<'_> {
        let (link, alias) = match inner.split_once('|') {
            // `\|` is how Obsidian escapes the pipe inside tables
            Some((link, alias)) => (link.strip_suffix('\\').unwrap_or(link), Some(alias.trim())),
            None => (inner, None),
        };
        let (target, heading) = match link.split_once('#') {
            Some((target, heading)) => (target.trim(), Some(heading.trim())),
            None => (link.trim(), None),
        };

        WikiLink {
            target,
            heading: heading.filter(|h| !h.is_empty()),
            alias: alias.filter(|a| !a.is_empty()),
        }
    }

    /// The text Obsidian shows for the link: the alias, or `note > heading`.
    pub fn display_text(&self) -> String {
        if let Some(alias) = self.alias {
            return alias.to_string();
        }
        match (self.target, self.heading) {
            ("", Some(heading)) => heading.to_string(),
            (target, Some(heading)) => format!("{target} > {heading}"),
            (target, None) => target.to_string(),
        }
    }

    pub fn heading_anchor(&self) -> Option<String> {
        self.heading.map(slugify)
    }
}

/// A `[[...]]` occurrence in some text.
#[derive(Debug, PartialEq, Eq)]
pub struct WikiLinkMatch<'a> {
    /// Byte range of the whole `[[...]]`, including a leading `!` for embeds.
    pub start: usize,
    pub end: usize,
    pub inner: &'a str,
    /// `![[...]]` embeds a note instead of linking to it.
    pub is_embed: bool,
}

pub fn find_wikilinks(text: &str) -> Vec<WikiLinkMatch<'_>> {
    let mut matches = Vec::new();
    let mut offset = 0;

    while let Some(open) = text[offset..].find("[[") {
        let inner_start = offset + open + 2;
        let Some(close) = text[inner_start..].find("]]") else {
            break;
        };
        let inner = &text[inner_start..inner_start + close];

        if inner.trim().is_empty() || inner.contains(['[', ']', '\n', '<', '>']) {
            offset += open + 1;
            continue;
        }

        let is_embed = text[..offset + open].ends_with('!');
        matches.push(WikiLinkMatch {
            start: offset + open - usize::from(is_embed),
            end: inner_start + close + 2,
            inner,
            is_embed,
        });
        offset = inner_start + close + 2;
    }

    matches
}

/// Lowercase, alphanumerics kept, runs of anything else collapsed into `-`.
pub fn slugify(text: &str) -> String {
    let mut slug = String::with_capacity(text.len());
    for c in text.trim().chars().flat_map(char::to_lowercase) {
        if c.is_alphanumeric() {
            slug.push(c);
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    slug.trim_end_matches('-').to_string()
}

/// The published posts, looked up the way Obsidian resolves `[[links]]`:
/// by full path, by partial path, or by bare note name, ignoring case.
#[derive(Default)]
pub struct LinkIndex {
    by_path: HashMap<String, String>,
    by_name: HashMap<String, Vec<String>>,
}

impl LinkIndex {
    /// `file_paths` are garden-relative, with the `.md` extension.
    pub fn new<I, S>(file_paths: I) -> LinkIndex
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut index = LinkIndex::default();
        for file_path in file_paths {
            let file_path = file_path.as_ref();
            let path = file_path.strip_suffix(".md").unwrap_or(file_path);
            let name = path.rsplit('/').next().unwrap_or(path);

            index.by_path.insert(path.to_lowercase(), path.to_string());
            index
                .by_name
                .entry(name.to_lowercase())
                .or_default()
                .push(path.to_string());
        }
        for paths in index.by_name.values_mut() {
            // Shortest path wins when several notes share a name, like in Obsidian
            paths.sort_by(|a, b| a.len().cmp(&b.len()).then_with(|| a.cmp(b)));
        }
        index
    }

    /// Whether `path` (without `.md`) is exactly a published post.
    pub fn contains(&self, path: &str) -> bool {
        self.by_path
            .get(&path.to_lowercase())
            .is_some_and(|found| found == path)
    }

    /// Garden-relative path (without `.md`) that a wikilink target points to.
    pub fn resolve(&self, target: &str) -> Option<&str> {
        let target = target
            .trim()
            .trim_start_matches("./")
            .trim_start_matches('/');
        let target = target.strip_suffix(".md").unwrap_or(target).to_lowercase();
        if target.is_empty() {
            return None;
        }
        if let Some(path) = self.by_path.get(&target) {
            return Some(path);
        }

        let name = target.rsplit('/').next().unwrap_or(&target);
        let suffix = format!("/{target}");
        self.by_name
            .get(name)?
            .iter()
            .find(|path| path.to_lowercase().ends_with(&suffix))
            .or_else(|| {
                self.by_name
                    .get(name)?
                    .first()
                    .filter(|_| !target.contains('/'))
            })
            .map(String::as_str)
    }

    /// Where a wikilink leads, if its note exists.
    pub fn wikilink_target(&self, link: &WikiLink) -> Option<LinkTarget> {
        Some(LinkTarget {
            path: self.resolve(link.target)?.to_string(),
            fragment: link.heading_anchor(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index() -> LinkIndex {
        LinkIndex::new([
            "welcome.md",
            "notes/fog.md",
            "journal/2025/kilbarrack.md",
            "archive/old/fog.md",
        ])
    }

    #[test]
    fn markdown_links_resolve_from_the_garden_root() {
        let target = |href| markdown_link_target(href).map(|t| (t.path, t.fragment));

        assert_eq!(target("hello"), Some(("hello".into(), None)));
        assert_eq!(target("./hello.md"), Some(("hello".into(), None)));
        assert_eq!(
            target("notes/fog#the-fog"),
            Some(("notes/fog".into(), Some("the-fog".into())))
        );
    }

    #[test]
    fn non_post_markdown_links_are_ignored() {
        for href in [
            "https://rust-lang.org/",
            "mailto:a@b.c",
            "/news",
            "#top",
            "?q=1",
            "",
        ] {
            assert_eq!(markdown_link_target(href), None, "{href:?}");
        }
    }

    #[test]
    fn link_targets_build_post_urls_and_file_paths() {
        let target = markdown_link_target("notes/fog#the-fog").unwrap();

        assert_eq!(target.url(), "/posts/notes/fog#the-fog");
        assert_eq!(target.file_path(), "notes/fog.md");
    }

    #[test]
    fn wikilinks_parse_target_heading_and_alias() {
        assert_eq!(
            WikiLink::parse("fog#The fog|that fog"),
            WikiLink {
                target: "fog",
                heading: Some("The fog"),
                alias: Some("that fog")
            }
        );
        assert_eq!(WikiLink::parse("fog\\|alias").alias, Some("alias"));
        assert_eq!(WikiLink::parse("fog\\|alias").target, "fog");
    }

    #[test]
    fn wikilink_text_falls_back_like_obsidian() {
        assert_eq!(WikiLink::parse("fog").display_text(), "fog");
        assert_eq!(WikiLink::parse("fog|Fog!").display_text(), "Fog!");
        assert_eq!(WikiLink::parse("fog#Intro").display_text(), "fog > Intro");
        assert_eq!(WikiLink::parse("#Intro").display_text(), "Intro");
    }

    #[test]
    fn finds_links_and_embeds_but_not_broken_brackets() {
        let text = "a [[fog]] b ![[welcome]] c [[]] d [[x\ny]] e [[[[nested]]";
        let found: Vec<(&str, bool)> = find_wikilinks(text)
            .iter()
            .map(|m| (m.inner, m.is_embed))
            .collect();

        assert_eq!(
            found,
            vec![("fog", false), ("welcome", true), ("nested", false)]
        );
        let embed = &find_wikilinks(text)[1];
        assert_eq!(&text[embed.start..embed.end], "![[welcome]]");
    }

    #[test]
    fn notes_resolve_by_name_path_or_partial_path_ignoring_case() {
        let index = index();

        assert_eq!(index.resolve("Kilbarrack"), Some("journal/2025/kilbarrack"));
        assert_eq!(
            index.resolve("journal/2025/kilbarrack.md"),
            Some("journal/2025/kilbarrack")
        );
        assert_eq!(
            index.resolve("2025/kilbarrack"),
            Some("journal/2025/kilbarrack")
        );
        assert_eq!(index.resolve("old/fog"), Some("archive/old/fog"));
        assert_eq!(index.resolve("missing"), None);
        assert_eq!(index.resolve("other/kilbarrack"), None);
    }

    #[test]
    fn shared_names_resolve_to_the_shortest_path() {
        assert_eq!(index().resolve("fog"), Some("notes/fog"));
    }

    #[test]
    fn contains_is_exact() {
        let index = index();

        assert!(index.contains("notes/fog"));
        assert!(!index.contains("Notes/Fog"));
        assert!(!index.contains("fog"));
    }

    #[test]
    fn headings_become_anchors() {
        assert_eq!(slugify("  The Fog, again! "), "the-fog-again");
        assert_eq!(
            index()
                .wikilink_target(&WikiLink::parse("fog#The Fog"))
                .unwrap()
                .url(),
            "/posts/notes/fog#the-fog"
        );
    }
}
