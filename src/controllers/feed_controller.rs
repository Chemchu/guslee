use actix_web::{HttpRequest, HttpResponse, get, web};
use chrono::NaiveDate;
use search_engine::utils::{DEFAULT_LANG, Post};

use crate::controllers::{AppState, base_url, encode_path};
use crate::i18n::{LANG_PARAM, Locale};

pub fn configure_services(cfg: &mut web::ServiceConfig) {
    cfg.service(rss_feed);
}

#[get("/rss.xml")]
async fn rss_feed(app_state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let locale = Locale::from_query(req.query_string()).unwrap_or_default();
    let posts = app_state
        .post_search_engine
        .published_posts(locale.as_str())
        .await;

    HttpResponse::Ok()
        .content_type("application/rss+xml; charset=utf-8")
        .body(rss(&app_state.app_name, &base_url(&req), locale, &posts))
}

fn lang_query(lang: &str) -> String {
    if lang == DEFAULT_LANG {
        String::new()
    } else {
        format!("?{LANG_PARAM}={lang}")
    }
}

fn rss(app_name: &str, base_url: &str, locale: Locale, posts: &[Post]) -> String {
    let lang = locale.as_str();
    let mut xml = format!(
        concat!(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n",
            "<rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\">\n",
            "<channel>\n",
            "<title>{title}</title>\n",
            "<link>{base}/</link>\n",
            "<description>{description}</description>\n",
            "<language>{lang}</language>\n",
            "<atom:link href=\"{base}/rss.xml{feed_query}\" rel=\"self\" type=\"application/rss+xml\"/>\n",
        ),
        title = escape_xml(app_name),
        base = escape_xml(base_url),
        description = escape_xml(locale.texts().site_description),
        lang = lang,
        feed_query = escape_xml(&lang_query(lang)),
    );

    for post in posts {
        let path = post
            .file_path
            .strip_suffix(".md")
            .unwrap_or(&post.file_path);
        let link = escape_xml(&format!(
            "{base_url}/posts/{}{}",
            encode_path(path),
            lang_query(&post.lang)
        ));

        xml.push_str("<item>\n");
        xml.push_str(&format!(
            "<title>{}</title>\n",
            escape_xml(&post.metadata.title)
        ));
        xml.push_str(&format!("<link>{link}</link>\n"));
        xml.push_str(&format!("<guid isPermaLink=\"true\">{link}</guid>\n"));
        xml.push_str(&format!(
            "<description>{}</description>\n",
            escape_xml(&post.metadata.description)
        ));
        if let Some(date) = rfc2822_date(&post.metadata.date) {
            xml.push_str(&format!("<pubDate>{date}</pubDate>\n"));
        }
        for tag in &post.metadata.tags {
            xml.push_str(&format!("<category>{}</category>\n", escape_xml(tag)));
        }
        xml.push_str("</item>\n");
    }

    xml.push_str("</channel>\n</rss>\n");
    xml
}

fn rfc2822_date(date: &str) -> Option<String> {
    let date = NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()?;
    Some(date.and_hms_opt(0, 0, 0)?.and_utc().to_rfc2822())
}

fn escape_xml(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use search_engine::utils::MdMetadata;

    fn post(file_path: &str, title: &str, date: &str) -> Post {
        Post {
            file_name: file_path.rsplit('/').next().unwrap().to_string(),
            file_path: file_path.to_string(),
            lang: DEFAULT_LANG.to_string(),
            metadata: MdMetadata {
                title: title.to_string(),
                topic: None,
                description: "A <b>bold</b> & \"quoted\" post".to_string(),
                tags: vec!["gaming".to_string()],
                date: date.to_string(),
                is_draft: false,
                post_source_url: String::new(),
                reading_time: 1,
                last_updated: None,
            },
            content: String::new(),
        }
    }

    #[test]
    fn feed_lists_posts_with_absolute_links() {
        let xml = rss(
            "Gus' garden",
            "https://g.dev",
            Locale::En,
            &[post(
                "journal/2025/my note.md",
                "Fog & letters",
                "2025-09-15",
            )],
        );

        assert!(
            xml.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<rss version=\"2.0\"")
        );
        assert!(xml.contains("<title>Gus&apos; garden</title>"));
        assert!(xml.contains("<language>en</language>"));
        assert!(xml.contains("<atom:link href=\"https://g.dev/rss.xml\" rel=\"self\""));
        assert!(xml.contains("<title>Fog &amp; letters</title>"));
        assert!(xml.contains("<link>https://g.dev/posts/journal/2025/my%20note</link>"));
        assert!(xml.contains(
            "<description>A &lt;b&gt;bold&lt;/b&gt; &amp; &quot;quoted&quot; post</description>"
        ));
        assert!(xml.contains("<pubDate>Mon, 15 Sep 2025 00:00:00 +0000</pubDate>"));
        assert!(xml.contains("<category>gaming</category>"));
        assert!(xml.ends_with("</channel>\n</rss>\n"));
    }

    #[test]
    fn invalid_dates_leave_out_the_pub_date() {
        let xml = rss(
            "G",
            "https://g.dev",
            Locale::En,
            &[post("a.md", "A", "someday")],
        );

        assert!(!xml.contains("<pubDate>"));
        assert!(xml.contains("<title>A</title>"));
    }

    #[test]
    fn translated_feeds_link_to_the_translated_posts() {
        let translated = Post {
            lang: "es".to_string(),
            ..post("notes/fog.md", "Niebla", "2025-03-01")
        };
        let untranslated = post("notes/letters.md", "Letters", "2025-02-01");

        let xml = rss(
            "G",
            "https://g.dev",
            Locale::Es,
            &[translated, untranslated],
        );

        assert!(xml.contains("<language>es</language>"));
        assert!(xml.contains(
            "<description>Mi jardín digital personal, hecho con Rust + HTMX</description>"
        ));
        assert!(xml.contains("<atom:link href=\"https://g.dev/rss.xml?lang=es\" rel=\"self\""));
        assert!(xml.contains("<link>https://g.dev/posts/notes/fog?lang=es</link>"));
        assert!(
            xml.contains("<guid isPermaLink=\"true\">https://g.dev/posts/notes/fog?lang=es</guid>")
        );
        assert!(xml.contains("<link>https://g.dev/posts/notes/letters</link>"));
    }
}
