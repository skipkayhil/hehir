use worker::{Context, Env, event};

#[event(fetch)]
async fn fetch(req: http::Request<worker::Body>, _env: Env, _ctx: Context) -> HehirResponse {
    route(req)
}

type HehirResponse = http::Result<http::Response<http_body_util::Full<bytes::Bytes>>>;

fn route(_req: http::Request<worker::Body>) -> HehirResponse {
    http::Response::builder()
        .status(http::StatusCode::OK)
        .body(HTML.into())
}

const HTML: &str = r#"
<!DOCTYPE html>
    <head>
        <meta charset="utf-8">
        <style>
            body {
                background: #181818;
                color: #d8d8d8;
                font-family: system-ui, sans-serif;
                max-width: 70ch;
                padding: 2ch;
                margin: auto;
                line-height: 1.75;
            }

            a {
                color: #9fd5fc;
            }
        </style>
    </head>
    <body>
        <h1>hehir</h1>

        <p>
            /hɛ(ə)ɹ/
        </p>

        <p>
            A <a href="">bunny</a> implementation.
        </p>
    </body>
</html>
"#;
