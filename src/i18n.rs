use std::convert::Infallible;
use std::future::{Ready, ready};

use actix_web::{
    Error, FromRequest, HttpRequest,
    body::MessageBody,
    cookie::{Cookie, SameSite, time::Duration},
    dev::{Payload, ServiceRequest, ServiceResponse},
    http::header::{ACCEPT_LANGUAGE, HeaderValue, VARY},
    middleware::Next,
};

pub use texts::{Texts, fill};

mod en;
mod es;
mod texts;

pub const LANG_PARAM: &str = "lang";
const STATIC_PREFIX: &str = "/_static";

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Locale {
    #[default]
    En,
    Es,
}

impl Locale {
    pub const ALL: [Locale; 2] = [Locale::En, Locale::Es];

    pub fn as_str(self) -> &'static str {
        match self {
            Locale::En => "en",
            Locale::Es => "es",
        }
    }

    pub fn texts(self) -> &'static Texts {
        match self {
            Locale::En => &en::TEXTS,
            Locale::Es => &es::TEXTS,
        }
    }

    pub fn from_tag(tag: &str) -> Option<Locale> {
        let primary = tag.trim().split(['-', '_']).next()?;
        Locale::ALL
            .into_iter()
            .find(|locale| locale.as_str().eq_ignore_ascii_case(primary))
    }

    pub fn from_accept_language(header: &str) -> Option<Locale> {
        let mut best: Option<(Locale, f32)> = None;

        for entry in header.split(',') {
            let mut parts = entry.split(';');
            let Some(locale) = parts.next().and_then(Locale::from_tag) else {
                continue;
            };
            let quality = parts
                .find_map(|param| param.trim().strip_prefix("q="))
                .map_or(Some(1.0), |q| q.trim().parse::<f32>().ok());

            match quality {
                Some(q) if q > 0.0 && best.is_none_or(|(_, best_q)| q > best_q) => {
                    best = Some((locale, q))
                }
                _ => {}
            }
        }

        best.map(|(locale, _)| locale)
    }

    pub fn from_query(query: &str) -> Option<Locale> {
        query
            .split('&')
            .filter_map(|pair| pair.split_once('='))
            .find(|(key, _)| *key == LANG_PARAM)
            .and_then(|(_, value)| Locale::from_tag(value))
    }

    pub fn negotiate(req: &HttpRequest) -> Locale {
        Locale::from_query(req.query_string())
            .or_else(|| {
                req.cookie(LANG_PARAM)
                    .and_then(|cookie| Locale::from_tag(cookie.value()))
            })
            .or_else(|| {
                req.headers()
                    .get(ACCEPT_LANGUAGE)
                    .and_then(|value| value.to_str().ok())
                    .and_then(Locale::from_accept_language)
            })
            .unwrap_or_default()
    }

    pub fn cookie(self) -> Cookie<'static> {
        Cookie::build(LANG_PARAM, self.as_str())
            .path("/")
            .max_age(Duration::days(365))
            .same_site(SameSite::Lax)
            .finish()
    }
}

pub async fn locale_headers(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    if req.path().starts_with(STATIC_PREFIX) {
        return next.call(req).await;
    }

    let chosen = Locale::from_query(req.query_string());
    let mut res = next.call(req).await?;
    res.headers_mut()
        .append(VARY, HeaderValue::from_static("Accept-Language, Cookie"));
    if let Some(locale) = chosen {
        res.response_mut().add_cookie(&locale.cookie())?;
    }
    Ok(res)
}

impl FromRequest for Locale {
    type Error = Infallible;
    type Future = Ready<Result<Locale, Infallible>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        ready(Ok(Locale::negotiate(req)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{
        App, HttpResponse,
        http::header::SET_COOKIE,
        middleware::from_fn,
        test::{self, TestRequest},
        web,
    };

    #[test]
    fn default_locale_matches_the_search_engine_default() {
        assert_eq!(
            Locale::default().as_str(),
            search_engine::utils::DEFAULT_LANG
        );
    }

    #[test]
    fn tags_match_on_the_primary_subtag() {
        assert_eq!(Locale::from_tag("es"), Some(Locale::Es));
        assert_eq!(Locale::from_tag("es-ES"), Some(Locale::Es));
        assert_eq!(Locale::from_tag("ES_mx"), Some(Locale::Es));
        assert_eq!(Locale::from_tag(" en-GB "), Some(Locale::En));
        assert_eq!(Locale::from_tag("fr"), None);
        assert_eq!(Locale::from_tag(""), None);
    }

    #[test]
    fn accept_language_picks_the_highest_quality_supported_locale() {
        assert_eq!(
            Locale::from_accept_language("es-ES,es;q=0.9,en;q=0.8"),
            Some(Locale::Es)
        );
        assert_eq!(
            Locale::from_accept_language("fr-FR,fr;q=0.9,en;q=0.5,es;q=0.7"),
            Some(Locale::Es)
        );
        assert_eq!(
            Locale::from_accept_language("en;q=0.4, es ; q=0.6"),
            Some(Locale::Es)
        );
    }

    #[test]
    fn accept_language_ties_keep_the_first_listed_locale() {
        assert_eq!(Locale::from_accept_language("en,es"), Some(Locale::En));
        assert_eq!(Locale::from_accept_language("es,en"), Some(Locale::Es));
    }

    #[test]
    fn accept_language_ignores_rejected_unknown_and_malformed_entries() {
        assert_eq!(
            Locale::from_accept_language("es;q=0,en;q=0.1"),
            Some(Locale::En)
        );
        assert_eq!(
            Locale::from_accept_language("es;q=abc,en;q=0.2"),
            Some(Locale::En)
        );
        assert_eq!(Locale::from_accept_language("fr,de,*"), None);
        assert_eq!(Locale::from_accept_language(",;;,"), None);
        assert_eq!(Locale::from_accept_language(""), None);
    }

    #[test]
    fn query_lang_param_is_read_from_the_query_string() {
        assert_eq!(Locale::from_query("lang=es"), Some(Locale::Es));
        assert_eq!(Locale::from_query("q=fog&lang=es-ES"), Some(Locale::Es));
        assert_eq!(Locale::from_query("language=es"), None);
        assert_eq!(Locale::from_query("lang=fr"), None);
        assert_eq!(Locale::from_query(""), None);
    }

    #[test]
    fn requests_without_hints_fall_back_to_english() {
        let req = TestRequest::get().to_http_request();
        assert_eq!(Locale::negotiate(&req), Locale::En);

        let req = TestRequest::get()
            .insert_header((ACCEPT_LANGUAGE, "fr-FR,de;q=0.5"))
            .to_http_request();
        assert_eq!(Locale::negotiate(&req), Locale::En);
    }

    #[test]
    fn query_beats_cookie_and_cookie_beats_header() {
        let req = TestRequest::get()
            .uri("/posts/welcome?lang=en")
            .cookie(Cookie::new(LANG_PARAM, "es"))
            .insert_header((ACCEPT_LANGUAGE, "es"))
            .to_http_request();
        assert_eq!(Locale::negotiate(&req), Locale::En);

        let req = TestRequest::get()
            .cookie(Cookie::new(LANG_PARAM, "en"))
            .insert_header((ACCEPT_LANGUAGE, "es"))
            .to_http_request();
        assert_eq!(Locale::negotiate(&req), Locale::En);

        let req = TestRequest::get()
            .cookie(Cookie::new(LANG_PARAM, "klingon"))
            .insert_header((ACCEPT_LANGUAGE, "es"))
            .to_http_request();
        assert_eq!(Locale::negotiate(&req), Locale::Es);
    }

    #[actix_web::test]
    async fn locale_is_extracted_from_the_request() {
        let (req, mut payload) = TestRequest::get()
            .insert_header((ACCEPT_LANGUAGE, "es-ES,es;q=0.9"))
            .to_http_parts();
        let locale = Locale::from_request(&req, &mut payload).await.unwrap();
        assert_eq!(locale, Locale::Es);
    }

    async fn call(uri: &str) -> ServiceResponse {
        let app = test::init_service(
            App::new()
                .wrap(from_fn(locale_headers))
                .default_service(web::to(HttpResponse::Ok)),
        )
        .await;
        test::call_service(&app, TestRequest::get().uri(uri).to_request())
            .await
            .map_into_boxed_body()
    }

    #[actix_web::test]
    async fn pages_vary_on_the_language_hints() {
        let res = call("/posts/welcome").await;

        assert_eq!(res.headers().get(VARY).unwrap(), "Accept-Language, Cookie");
        assert!(res.headers().get(SET_COOKIE).is_none());
    }

    #[actix_web::test]
    async fn choosing_a_language_in_the_query_remembers_it_in_a_cookie() {
        let res = call("/posts/welcome?lang=es").await;

        let cookie = res.headers().get(SET_COOKIE).unwrap().to_str().unwrap();
        assert!(cookie.starts_with("lang=es;"));
        assert!(cookie.contains("Path=/"));
        assert!(cookie.contains("SameSite=Lax"));
        assert!(cookie.contains("Max-Age=31536000"));
    }

    #[actix_web::test]
    async fn unsupported_query_languages_set_no_cookie() {
        let res = call("/posts/welcome?lang=klingon").await;

        assert!(res.headers().get(SET_COOKIE).is_none());
    }

    #[actix_web::test]
    async fn static_files_are_left_alone() {
        let res = call("/_static/htmx.min.js?lang=es").await;

        assert!(res.headers().get(VARY).is_none());
        assert!(res.headers().get(SET_COOKIE).is_none());
    }
}
