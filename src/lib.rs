use worker::{Context, Env, event};

#[event(fetch)]
async fn fetch(req: http::Request<worker::Body>, _env: Env, _ctx: Context) -> HehirResponse {
    route(req)
}

type HehirResponse = http::Result<
    http::Response<
        http_body_util::Either<
            http_body_util::Full<bytes::Bytes>,
            http_body_util::Empty<bytes::Bytes>,
        >,
    >,
>;

fn route(req: http::Request<worker::Body>) -> HehirResponse {
    match req.uri().path() {
        "/search" => {
            let query = req.uri().query().unwrap_or("");

            hop(query)
        }
        _ => http::Response::builder()
            .status(http::StatusCode::OK)
            .body(http_body_util::Either::Left(HTML.into())),
    }
}

fn hop(cmd: &str) -> HehirResponse {
    http::Response::builder()
        .status(http::StatusCode::SEE_OTHER)
        .header(
            http::header::LOCATION,
            Command::from(
                percent_encoding::percent_decode(cmd.as_bytes())
                    .decode_utf8()
                    .unwrap()
                    .as_ref(),
            )
            .to_location(),
        )
        .body(http_body_util::Either::Right(http_body_util::Empty::new()))
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

enum Command {
    Github(Option<String>),
    Google(String),
}

impl Command {
    fn to_location(&self) -> String {
        match self {
            Command::Github(Some(r)) => format!("https://github.com/{r}"),
            Command::Github(None) => "https://github.com/".to_string(),
            Command::Google(q) => format!("https://google.com/search?q={q}"),
        }
    }
}

impl From<&str> for Command {
    fn from(s: &str) -> Self {
        let end = s.find(" ");
        let cmd = &s[..end.unwrap_or(s.len())];

        match cmd {
            "gh" => match end {
                Some(e) => Self::Github(Some(s[(e + 1)..].to_string())),
                None => Self::Github(None),
            },
            _ => Self::Google(s.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_parses_gh() {
        let command: Command = "gh".into();

        assert!(matches!(command, Command::Github(s) if s == None));
    }

    #[test]
    fn command_parses_gh_with_arg() {
        let command: Command = "gh skipkayhil/hehir".into();

        assert!(matches!(command, Command::Github(s) if s == Some("skipkayhil/hehir".to_string())));
    }

    #[test]
    fn command_falls_back_to_google() {
        let command: Command = "g skipkayhil/hehir".into();

        assert!(matches!(command, Command::Google(s) if s == "g skipkayhil/hehir"));
    }
}
