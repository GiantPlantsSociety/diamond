use chrono::NaiveDateTime;

use actix_web::error::ErrorInternalServerError;
use actix_web::web::{Data, Json};
use actix_web::{dev, Error, FromRequest, HttpMessage, HttpRequest, HttpResponse};
use futures::future::{ready, FutureExt, LocalBoxFuture};
use serde::*;
use std::path::PathBuf;
use std::str::FromStr;
use whisper::interval::Interval;

use crate::context::Context;
use crate::error::{ParseError, ResponseError};
use crate::parse::{de_time_parse, time_parse};
use crate::storage::{RenderPoint, Walker};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct RenderQuery {
    target: Vec<String>,
    format: RenderFormat,
    #[serde(deserialize_with = "de_time_parse")]
    from: u32,
    #[serde(deserialize_with = "de_time_parse")]
    until: u32,
}

impl FromStr for RenderQuery {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let raw: Vec<(String, String)> = serde_urlencoded::from_str(s)?;

        let mut q = RenderQuery::default();
        for (key, value) in raw {
            match key.as_str() {
                "target" => q.target.push(value),
                "format" => q.format = value.parse()?,
                "from" => q.from = time_parse(value)?,
                "until" => q.until = time_parse(value)?,
                _ => {}
            };
        }

        Ok(q)
    }
}

impl FromRequest for RenderQuery {
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;
    type Config = ();

    fn from_request(req: &HttpRequest, payload: &mut dev::Payload) -> Self::Future {
        match req.content_type().to_lowercase().as_str() {
            "application/x-www-form-urlencoded" => String::from_request(req, payload)
                .map(|r| {
                    Ok(r?
                        .parse::<RenderQuery>()
                        .map_err(ErrorInternalServerError)?)
                })
                .boxed_local(),
            "application/json" => Json::<RenderQuery>::from_request(req, payload)
                .map(|r| Ok(r?.into_inner()))
                .boxed_local(),
            _ => ready(req.query_string().parse().map_err(ErrorInternalServerError)).boxed_local(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RenderFormat {
    Png,
    Raw,
    Csv,
    Json,
    Svg,
    Pdf,
    Dygraph,
    Rickshaw,
}

impl Default for RenderFormat {
    fn default() -> Self {
        RenderFormat::Json
    }
}

impl FromStr for RenderFormat {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "png" => Ok(RenderFormat::Png),
            "raw" => Ok(RenderFormat::Raw),
            "csv" => Ok(RenderFormat::Csv),
            "json" => Ok(RenderFormat::Json),
            "svg" => Ok(RenderFormat::Svg),
            "pdf" => Ok(RenderFormat::Pdf),
            "dygraph" => Ok(RenderFormat::Dygraph),
            "rickshaw" => Ok(RenderFormat::Rickshaw),
            _ => Err(ParseError::RenderFormat),
        }
    }
}

impl Display for RenderFormat {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let s = format!("{:?}", self);
        write!(f, "{}", s.to_ascii_lowercase())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderMetric(String);

#[derive(Debug, PartialEq)]
pub struct RenderPath(PathBuf);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderResponseEntry {
    target: String,
    datapoints: Vec<RenderPoint>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderResponse {
    entries: Vec<Option<RenderResponseEntry>>,
}

pub async fn render_handler<T: Walker>(
    ctx: Data<Context<T>>,
    query: RenderQuery,
) -> Result<HttpResponse, Error> {
    let interval = Interval::new(query.from, query.until).map_err(ResponseError::Kind)?;
    let format = query.format;

    let response: Result<Vec<RenderResponseEntry>, ResponseError> = query
        .target
        .into_iter()
        .map(|metric| {
            ctx.walker
                .walk(&metric, interval)
                .map(|points| RenderResponseEntry {
                    datapoints: points,
                    target: metric,
                })
        })
        .collect();

    Ok(response.map(|r| format_response(r, format))?)
}

// TODO: Extract code BELOW to response_format_csv module
trait IntoCsv {
    fn into_csv(self) -> String;
}

impl<T: IntoCsv> IntoCsv for Vec<T> {
    fn into_csv(self) -> String {
        self.into_iter()
            .map(|item| item.into_csv())
            .collect::<Vec<String>>()
            .join("\n")
            + "\n"
    }
}

impl IntoCsv for RenderResponseEntry {
    fn into_csv(self) -> String {
        let metric = self.target;
        let lines: Vec<String> = self
            .datapoints
            .into_iter()
            .map(|point| {
                let RenderPoint(val, ts) = point;
                let v = val.map(|f| format!("{}", f)).unwrap_or_default();
                let t = NaiveDateTime::from_timestamp(i64::from(ts), 0).format("%Y-%m-%d %H:%M:%S");
                // TODO: Use `csv` crate instead of "manual" string formatting
                format!("{},{},{}", metric, t, v)
            })
            .collect();
        lines.join("\n")
    }
}
// TODO: Extract code ABOVE to response_format_csv module

fn format_response(response: Vec<RenderResponseEntry>, format: RenderFormat) -> HttpResponse {
    match format {
        RenderFormat::Json => HttpResponse::Ok().json(response),
        RenderFormat::Csv => HttpResponse::Ok()
            .content_type("text/csv")
            .body(response.into_csv()),
        _ => HttpResponse::BadRequest()
            .content_type("text/plain")
            .body(format!("Format '{}' not supported", format)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opts::Args;
    use crate::test_utils::WalkerConst;

    use actix_web::http::header::{CONTENT_LENGTH, CONTENT_TYPE};
    use actix_web::http::StatusCode;
    use actix_web::test::TestRequest;
    use futures::stream::StreamExt;
    use std::error;
    use std::time::{SystemTime, UNIX_EPOCH};

    async fn render_response(
        ctx: Context<WalkerConst>,
        query: RenderQuery,
    ) -> (StatusCode, String, String) {
        let mut response: HttpResponse = render_handler(Data::new(ctx), query).await.ok().unwrap();
        let content_type: String = response
            .head()
            .headers()
            .get(CONTENT_TYPE)
            .unwrap()
            .clone()
            .to_str()
            .unwrap()
            .to_string();
        let status = response.status();

        let body = response
            .take_body()
            .into_body::<Vec<u8>>()
            .map(|x| x.unwrap().to_vec())
            .concat()
            .await;
        (status, content_type, String::from_utf8(body).unwrap())
    }

    #[actix_rt::test]
    async fn render_handler_unsupported() {
        let formats: Vec<RenderFormat> = vec![
            RenderFormat::Png,
            RenderFormat::Raw,
            RenderFormat::Svg,
            RenderFormat::Pdf,
            RenderFormat::Dygraph,
            RenderFormat::Rickshaw,
        ];
        for format in formats {
            let ctx = Context {
                args: Args {
                    path: PathBuf::new(),
                    force: false,
                    port: 0,
                },
                walker: WalkerConst(vec![]),
            };
            let query = RenderQuery {
                target: vec![],
                format: format.clone(),
                from: 0,
                until: 0,
            };
            let (status, ct, response) = render_response(ctx, query).await;
            assert_eq!(status, StatusCode::BAD_REQUEST);
            assert_eq!(ct, "text/plain");
            assert_eq!(response, format!("Format '{}' not supported", format));
        }
    }

    #[actix_rt::test]
    async fn render_handler_json_ok_empty() {
        let ctx = Context {
            args: Args {
                path: PathBuf::new(),
                force: false,
                port: 0,
            },
            walker: WalkerConst(vec![]),
        };
        let query = RenderQuery {
            target: vec![],
            format: RenderFormat::Json,
            from: 0,
            until: 0,
        };
        let (status, ct, response) = render_response(ctx, query).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(ct, "application/json");
        assert_eq!(response, "[]");
    }

    #[actix_rt::test]
    async fn render_handler_json_ok_full() {
        let t = 1_564_432_988;
        let ctx = Context {
            args: Args {
                path: PathBuf::new(),
                force: false,
                port: 0,
            },
            walker: WalkerConst(vec![
                RenderPoint(Some(1.0 as f64), t),
                RenderPoint(None, t + 10),
                RenderPoint(Some(2.0 as f64), t + 100),
                RenderPoint(Some(3.0 as f64), t + 1000),
            ]),
        };
        let query = RenderQuery {
            target: vec!["i.am.a.metric".to_owned()],
            format: RenderFormat::Json,
            from: 0,
            until: 0,
        };
        let (status, ct, response) = render_response(ctx, query).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(ct, "application/json");
        assert_eq!(
                response,
                "[{\"target\":\"i.am.a.metric\",\"datapoints\":[[1.0,1564432988],[null,1564432998],[2.0,1564433088],[3.0,1564433988]]}]"
            )
    }

    #[actix_rt::test]
    async fn render_handler_csv_ok_empty() {
        let ctx = Context {
            args: Args {
                path: PathBuf::new(),
                force: false,
                port: 0,
            },
            walker: WalkerConst(vec![]),
        };
        let query = RenderQuery {
            target: vec![],
            format: RenderFormat::Csv,
            from: 0,
            until: 0,
        };
        let (status, ct, response) = render_response(ctx, query).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(ct, "text/csv");
        assert_eq!(response, "\n")
    }

    #[actix_rt::test]
    async fn render_handler_csv_ok_full() {
        let t = 1_564_432_988;
        let ctx = Context {
            args: Args {
                path: PathBuf::new(),
                force: false,
                port: 0,
            },
            walker: WalkerConst(vec![
                RenderPoint(Some(1.1 as f64), t),
                RenderPoint(Some(2.2 as f64), t + 60),
                RenderPoint(None, t + 60 * 60),
                RenderPoint(Some(3.3 as f64), t + 24 * 60 * 60),
            ]),
        };
        let query = RenderQuery {
            target: vec!["i.am.a.metric".to_owned()],
            format: RenderFormat::Csv,
            from: 0,
            until: 0,
        };
        let (status, ct, response) = render_response(ctx, query).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(ct, "text/csv");
        assert_eq!(
                response,
                "i.am.a.metric,2019-07-29 20:43:08,1.1\ni.am.a.metric,2019-07-29 20:44:08,2.2\ni.am.a.metric,2019-07-29 21:43:08,\ni.am.a.metric,2019-07-30 20:43:08,3.3\n"
            )
    }

    #[test]
    fn url_deserialize_one() -> Result<(), Error> {
        let params = RenderQuery {
            format: RenderFormat::Json,
            target: ["app.numUsers".to_owned()].to_vec(),
            from: 0,
            until: 10,
        };

        assert_eq!(
            "target=app.numUsers&format=json&from=0&until=10".parse::<RenderQuery>()?,
            params
        );

        Ok(())
    }

    #[test]
    fn url_deserialize_multiple() -> Result<(), Error> {
        let params = RenderQuery {
            format: RenderFormat::Json,
            target: ["app.numUsers".to_owned(), "app.numServers".to_owned()].to_vec(),
            from: 0,
            until: 10,
        };

        assert_eq!(
            "target=app.numUsers&target=app.numServers&format=json&from=0&until=10"
                .parse::<RenderQuery>()?,
            params
        );

        Ok(())
    }

    #[test]
    fn url_deserialize_none() -> Result<(), Error> {
        let params = RenderQuery {
            format: RenderFormat::Json,
            target: Vec::new(),
            from: 0,
            until: 10,
        };

        assert_eq!(
            "format=json&from=0&until=10".parse::<RenderQuery>()?,
            params
        );

        Ok(())
    }

    #[test]
    fn url_deserialize_other() -> Result<(), Error> {
        let params = RenderQuery {
            format: RenderFormat::Json,
            target: vec!["m1".to_owned()],
            from: 0,
            until: 10,
        };

        assert_eq!(
            "target=m1&format=json&from=0&until=10&other=x".parse::<RenderQuery>()?,
            params
        );

        Ok(())
    }

    #[test]
    fn url_deserialize_time_yesterday_now() -> Result<(), Box<dyn error::Error>> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32;

        let params = RenderQuery {
            format: RenderFormat::Json,
            target: ["m1".to_owned(), "m2".to_owned()].to_vec(),
            from: now - 3600 * 24,
            until: now,
        };

        assert_eq!(
            "target=m1&target=m2&format=json&from=yesterday&until=now".parse::<RenderQuery>()?,
            params
        );

        Ok(())
    }

    #[test]
    fn url_deserialize_time_relative() -> Result<(), Box<dyn error::Error>> {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32;

        let params = RenderQuery {
            format: RenderFormat::Json,
            target: ["m1".to_owned(), "m2".to_owned()].to_vec(),
            from: now - 5 * 3600 * 24,
            until: now - 5 * 60,
        };

        assert_eq!(
            "target=m1&target=m2&format=json&from=-5d&until=-5min".parse::<RenderQuery>()?,
            params
        );

        let params2 = RenderQuery {
            format: RenderFormat::Json,
            target: ["m1".to_owned(), "m2".to_owned()].to_vec(),
            from: now - 5 * 3600 * 24 * 7,
            until: now - 5,
        };

        assert_eq!(
            "target=m1&target=m2&format=json&from=-5w&until=-5s".parse::<RenderQuery>()?,
            params2
        );

        let params3 = RenderQuery {
            format: RenderFormat::Json,
            target: ["m1".to_owned(), "m2".to_owned()].to_vec(),
            from: now - 2 * 3600 * 24 * 365,
            until: now - 5 * 3600,
        };

        assert_eq!(
            "target=m1&target=m2&format=json&from=-2y&until=-5h".parse::<RenderQuery>()?,
            params3
        );

        let params4 = RenderQuery {
            format: RenderFormat::Json,
            target: ["m1".to_owned(), "m2".to_owned()].to_vec(),
            from: now - 5 * 3600 * 24 * 30,
            until: now - 60,
        };

        assert_eq!(
            "target=m1&target=m2&format=json&from=-5mon&until=-1min".parse::<RenderQuery>()?,
            params4
        );

        Ok(())
    }

    #[test]
    fn url_deserialize_time_fail() -> Result<(), Error> {
        assert!("target=m1&target=m2&format=json&from=-&until=now"
            .parse::<RenderQuery>()
            .is_err());

        assert!("target=m1&target=m2&format=json&from=-d&until=now"
            .parse::<RenderQuery>()
            .is_err());

        assert!("target=m1&target=m2&format=json&from=&until=now"
            .parse::<RenderQuery>()
            .is_err());

        assert!("target=m1&target=m2&format=json&from=tomorrow&until=now"
            .parse::<RenderQuery>()
            .is_err());

        Ok(())
    }

    #[test]
    fn render_response_json() -> Result<(), Error> {
        let rd = serde_json::to_string(&[RenderResponseEntry {
            target: "entries".into(),
            datapoints: vec![
                RenderPoint(Some(1.0), 1_311_836_008),
                RenderPoint(Some(2.0), 1_311_836_009),
                RenderPoint(Some(3.0), 1_311_836_010),
                RenderPoint(Some(5.0), 1_311_836_011),
                RenderPoint(Some(6.0), 1_311_836_012),
                RenderPoint(None, 1_311_836_013),
            ],
        }])?;

        let rs = r#"[{"target":"entries","datapoints":[[1.0,1311836008],[2.0,1311836009],[3.0,1311836010],[5.0,1311836011],[6.0,1311836012],[null,1311836013]]}]"#;

        assert_eq!(rd, rs);
        Ok(())
    }

    #[test]
    #[allow(clippy::unreadable_literal)]
    fn render_response_json_parse() -> Result<(), Error> {
        let rd = [RenderResponseEntry {
            target: "entries".into(),
            datapoints: [
                RenderPoint(Some(1.0), 1_311_836_008),
                RenderPoint(Some(2.0), 1_311_836_009),
                RenderPoint(Some(3.0), 1_311_836_010),
                RenderPoint(Some(5.0), 1_311_836_011),
                RenderPoint(Some(6.0), 1_311_836_012),
                RenderPoint(None, 1_311_836_013),
            ]
            .to_vec(),
        }]
        .to_vec();

        let rs: Vec<RenderResponseEntry> = serde_json::from_str(
            r#"[{"target":"entries","datapoints":[[1.0,1311836008],[2.0,1311836009],[3.0,1311836010],[5.0,1311836011],[6.0,1311836012],[null,1311836013]]}]"#,
        )?;

        assert_eq!(rd, rs);
        Ok(())
    }

    #[test]
    fn format_parse() {
        assert_eq!("png".parse(), Ok(RenderFormat::Png));
        assert_eq!("raw".parse(), Ok(RenderFormat::Raw));
        assert_eq!("csv".parse(), Ok(RenderFormat::Csv));
        assert_eq!("json".parse(), Ok(RenderFormat::Json));
        assert_eq!("svg".parse(), Ok(RenderFormat::Svg));
        assert_eq!("pdf".parse(), Ok(RenderFormat::Pdf));
        assert_eq!("dygraph".parse(), Ok(RenderFormat::Dygraph));
        assert_eq!("rickshaw".parse(), Ok(RenderFormat::Rickshaw));
        assert_eq!("".parse::<RenderFormat>(), Err(ParseError::RenderFormat));
    }

    #[actix_rt::test]
    async fn render_request_parse_url() -> Result<(), actix_web::error::Error> {
        let req = TestRequest::with_uri("/render?target=app.numUsers&format=json&from=0&until=10")
            .to_http_request();

        let params = RenderQuery {
            format: RenderFormat::Json,
            target: ["app.numUsers".to_owned()].to_vec(),
            from: 0,
            until: 10,
        };

        assert_eq!(RenderQuery::extract(&req).await?, params);
        Ok(())
    }

    #[actix_rt::test]
    async fn render_request_parse_form() -> Result<(), actix_web::error::Error> {
        let s = "target=app.numUsers&format=json&from=0&until=10";

        let (req, mut payload) =
            TestRequest::with_header(CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(CONTENT_LENGTH, s.len())
                .set_payload(s)
                .to_http_parts();

        let params = RenderQuery {
            format: RenderFormat::Json,
            target: ["app.numUsers".to_owned()].to_vec(),
            from: 0,
            until: 10,
        };

        let res = RenderQuery::from_request(&req, &mut payload).await?;

        assert_eq!(res, params);
        Ok(())
    }

    #[actix_rt::test]
    async fn render_request_parse_json() -> Result<(), actix_web::error::Error> {
        let s = r#"{ "target":["app.numUsers"],"format":"json","from":"0","until":"10"}"#;

        let (req, mut pl) = TestRequest::with_uri("/render")
            .header("content-type", "application/json")
            .header(CONTENT_LENGTH, s.len())
            .set_payload(s)
            //.to_http_request();
            .to_http_parts();

        let params = RenderQuery {
            format: RenderFormat::Json,
            target: ["app.numUsers".to_owned()].to_vec(),
            from: 0,
            until: 10,
        };

        assert_eq!(RenderQuery::from_request(&req, &mut pl).await?, params);
        Ok(())
    }
}
