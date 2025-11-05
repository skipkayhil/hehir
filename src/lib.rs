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
            if let Some(query) = req.uri().query() {
                let query = query.replace("+", " ");
                let query = percent_encoding::percent_decode(query.as_bytes())
                    .decode_utf8()
                    .unwrap();

                return hop(&query);
            }
        }
        "/opensearch.xml" => {
            return http::Response::builder()
                .status(http::StatusCode::OK)
                .header(
                    http::header::CONTENT_TYPE,
                    "application/opensearchdescription+xml",
                )
                .body(http_body_util::Either::Left(opensearch(req.uri()).into()));
        }
        _ => (),
    }

    http::Response::builder()
        .status(http::StatusCode::OK)
        .body(http_body_util::Either::Left(HTML.into()))
}

fn hop(cmd: &str) -> HehirResponse {
    http::Response::builder()
        .status(http::StatusCode::SEE_OTHER)
        .header(http::header::LOCATION, Command::from(cmd).to_location())
        .body(http_body_util::Either::Right(http_body_util::Empty::new()))
}

fn opensearch(uri: &http::uri::Uri) -> String {
    format!(
        r#"<OpenSearchDescription xmlns="http://a9.com/-/spec/opensearch/1.1/" xmlns:moz="http://www.mozilla.org/2006/browser/search/">
  <ShortName>Hehir</ShortName>
  <Description>A bunny implementation</Description>
  <InputEncoding>UTF-8</InputEncoding>
  <Url type="text/html" template="{}://{}/search?{{searchTerms}}"/>
</OpenSearchDescription>"#,
        uri.scheme_str().unwrap(),
        uri.authority().unwrap().as_str()
    )
}

const HTML: &str = r#"
<!DOCTYPE html>
    <head>
        <meta charset="utf-8">
        <link rel="search" type="application/opensearchdescription+xml" title="Hehir" href="/opensearch.xml" />
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
            code {
                background: #2b2e3b;
                border: 1px solid #474a58;
                border-radius: 3px;
                color: #cacedf;
                padding: 1px 5px;
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

        <h2>installation</h2>

        An <a href="https://developer.mozilla.org/en-US/docs/Web/XML/Guides/OpenSearch">OpenSearch</a>
        Description is provided to make it simple to add Hehir as a search engine in your browser.

        <h3>firefox</h3>

        <p>
        While on this page, start typing in the address bar and select <code>Hehir</code> in the
        Unified Search dropdown.
        </p>

        <p>
        Alternatively, you can right click the address bar and select <code>Add "Hehir"</code>.
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
            Command::Github(None) => "https://github.com".to_string(),
            Command::Google(q) => {
                let mut url =
                    url::Url::parse("https://google.com/search").expect("This URL is valid");
                url.query_pairs_mut().append_pair("q", q);

                url.into()
            }
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

        assert_eq!("https://github.com", command.to_location());
    }

    #[test]
    fn command_parses_gh_with_arg() {
        let command: Command = "gh skipkayhil/hehir".into();

        assert_eq!("https://github.com/skipkayhil/hehir", command.to_location());
    }

    #[test]
    fn command_falls_back_to_google() {
        let command: Command = "g skipkayhil/hehir".into();

        assert_eq!(
            "https://google.com/search?q=g+skipkayhil%2Fhehir",
            command.to_location()
        );
    }

    #[test]
    fn command_encodes_and_decodes_plus() {
        let command: Command = "plus+plus".into();

        assert_eq!(
            "https://google.com/search?q=plus%2Bplus",
            command.to_location()
        );
    }
}
