use crate::controllers::{AppState, render_page};
use crate::i18n::Locale;
use actix_web::{HttpRequest, Responder, get, web};
use maud::html;

pub fn configure_services(cfg: &mut web::ServiceConfig) {
    cfg.service(news_page);
}

#[get("/news")]
async fn news_page(
    app_state: web::Data<AppState>,
    req: HttpRequest,
    locale: Locale,
) -> impl Responder {
    let t = locale.texts();
    let news = app_state
        .post_search_engine
        .published_posts(locale.as_str())
        .await;

    let is_empty = news.is_empty();

    let template = html! {
    div
    class="p-4 md:p-6 lg:p-8 overflow-auto w-full"
    {
            div
            class="max-w-5xl mx-auto"
            {
                div
                class="mb-12"
                {
                    h1
                    class="text-5xl md:text-6xl font-bold bg-clip-text mb-4"
                    {
                        (t.news_title)
                    }
                    p
                    class="text-lg text-slate-600 dark:text-slate-400"
                    {
                        (t.news_subtitle)
                    }
                }

                @if is_empty {
                    div
                    class="text-center py-20"
                    {
                        div
                        class="inline-flex items-center justify-center w-20 h-20 rounded-full bg-slate-200 dark:bg-slate-700 mb-6"
                        {
                            svg
                            class="w-10 h-10 text-slate-400"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                            {
                                path
                                stroke-linecap="round"
                                stroke-linejoin="round"
                                stroke-width="2"
                                d="M19 20H5a2 2 0 01-2-2V6a2 2 0 012-2h10a2 2 0 012 2v1m2 13a2 2 0 01-2-2V7m2 13a2 2 0 002-2V9a2 2 0 00-2-2h-2m-4-3H9M7 16h6M7 8h6v4H7V8z"
                                {}
                            }
                        }
                        h3
                        class="text-xl font-semibold text-slate-900 dark:text-slate-100 mb-2"
                        {
                            (t.news_empty_title)
                        }
                        p
                        class="text-slate-600 dark:text-slate-400"
                        {
                            (t.news_empty_body)
                        }
                    }
                } @else {
                    div
                    class="space-y-6"
                    {
                        @for post in news {
                        @let n = &post.metadata;
                        article
                        class="group relative shadow-sm hover:shadow-xl transition-all duration-300 overflow-hidden border border-shade-color"
                        {
                            a
                            class="block p-6 md:p-8"
                            href=(format!("/posts/{}", post.file_path.replace(".md", "")))
                            hx-target="#main-section"
                            hx-trigger="click"
                            hx-swap="innerHTML transition:true"
                            {
                                div
                                class="absolute left-0 top-0 h-full w-1 bg-primary-color transform scale-y-0 group-hover:scale-y-100 transition-transform duration-300"
                                {}

                                div
                                class="flex flex-col md:flex-row md:items-start md:justify-between gap-4"
                                {
                                    div
                                    class="flex-1 space-y-3"
                                    {
                                        h2
                                        class="text-2xl md:text-3xl font-semibold text-bright-color group-hover:text-primary-color transition-colors duration-200"
                                        {
                                            (n.title)
                                        }

                                        p
                                        class="text-base md:text-lg text-slate-600 dark:text-slate-400 leading-relaxed"
                                        {
                                            (n.description)
                                        }

                                        @if !n.tags.is_empty() {
                                            div
                                            class="flex flex-wrap gap-2 pt-2"
                                            {
                                                @for tag in &n.tags {
                                                    span
                                                    class="px-3 py-1 text-xs font-medium bg-primary-color/10 text-primary-color rounded-full"
                                                    {
                                                        (tag)
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    div
                                    class="flex items-center gap-2 text-slate-500 dark:text-slate-400"
                                    {
                                        svg
                                        class="w-5 h-5"
                                        fill="none"
                                        stroke="currentColor"
                                        viewBox="0 0 24 24"
                                        {
                                            path
                                            stroke-linecap="round"
                                            stroke-linejoin="round"
                                            stroke-width="2"
                                            d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
                                            {}
                                        }
                                        time
                                        class="text-sm md:text-base font-medium whitespace-nowrap"
                                        {
                                            (n.date)
                                        }
                                    }
                                }

                                div
                                class="flex items-center gap-2 mt-4 text-primary-color font-medium group-hover:gap-3 transition-all duration-200"
                                {
                                    span
                                    class="text-sm"
                                    {
                                        (t.news_read_more)
                                    }
                                    svg
                                    class="w-4 h-4 transform group-hover:translate-x-1 transition-transform duration-200"
                                    fill="none"
                                    stroke="currentColor"
                                    viewBox="0 0 24 24"
                                    {
                                        path
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        stroke-width="2"
                                        d="M9 5l7 7-7 7"
                                        {}
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }};

    render_page(&app_state, &req, &template.0)
}
