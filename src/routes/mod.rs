use actix_web::{
    HttpRequest, Responder, get,
    web::{self, Html},
};
use std::{fs, io};

/* fn get_markdown_options() -> Options {
    Options {
        parse: ParseOptions::gfm(), // GitHub Flavored Markdown
        compile: CompileOptions {
            allow_dangerous_html: true,
            allow_dangerous_protocol: false,
            ..CompileOptions::default()
        },
    }
} */

#[get("/")]
pub async fn landing() -> impl Responder {
    let path = "./templates/index.html";
    let content: String =
        fs::read_to_string(path).unwrap_or_else(|_| panic!("File not found: {}", path));

    Html::new(content)
}

#[get("/{post:.*}")]
pub async fn post(req: HttpRequest, route: web::Path<String>) -> impl Responder {
    let content: io::Result<String> = fs::read_to_string(format!("./garden/{}.md", route));

    /*     let options = get_markdown_options(); */

    let is_htmx_req = req.headers().get("HX-Request").is_some();
    if is_htmx_req {
        Html::new(match content {
            /*             Ok(md) => markdown::to_html_with_options(&md, &options).unwrap(), */
            Ok(md) => markdown::to_html(&md),
            Err(_err) => String::from("Fallback page"),
        })
    } else {
        Html::new(match content {
            /* Ok(md) => wrap_markdown_with_whole_page(
                &markdown::to_html_with_options(&md, &options).unwrap(),
            ), */
            Ok(md) => wrap_markdown_with_whole_page(&markdown::to_html(&md)),
            Err(_err) => String::from("Fallback page"),
        })
    }
}

fn wrap_markdown_with_whole_page(content: &str) -> String {
    let path = "./templates/index.html";
    let html: String =
        fs::read_to_string(path).unwrap_or_else(|_| panic!("File not found: {}", path));

    html.replace("{{CONTENT}}", content)
}
