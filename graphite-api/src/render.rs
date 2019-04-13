use actix_web::{HttpResponse, Json, State};
use failure::*;
use serde::*;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use whisper::interval::Interval;
use whisper::WhisperFile;

use actix_web::{FromRequest, HttpRequest};
use serde::{Deserialize, Deserializer};
use std::str::FromStr;

use crate::error::ParseError;
use crate::opts::*;
use actix_web::HttpMessage;
use futures::future::Future;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderQuery {
    target: Vec<String>,
    format: RenderFormat,
    #[serde(deserialize_with = "de_time_parse")]
    from: u32,
    #[serde(deserialize_with = "de_time_parse")]
    until: u32,
}

impl Default for RenderQuery {
    fn default() -> Self {
        RenderQuery {
            target: Vec::new(),
            format: RenderFormat::Json,
            from: 0,
            until: 0,
        }
    }
}

impl FromStr for RenderQuery {
    type Err = Error;

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

impl<S: 'static> FromRequest<S> for RenderQuery {
    type Config = ();
    type Result = Result<Self, actix_web::error::Error>;

    fn from_request(req: &HttpRequest<S>, _cfg: &Self::Config) -> Self::Result {
        match req.content_type().to_lowercase().as_str() {
            "application/x-www-form-urlencoded" => Ok(String::extract(req)?.wait()?.parse()?),
            "application/json" => Ok(Json::<RenderQuery>::extract(req).wait()?.into_inner()),
            _ => Ok(req.query_string().parse()?),
        }
    }
}

pub fn de_time_parse<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    time_parse(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
}

pub fn time_parse(s: String) -> Result<u32, ParseError> {
    if s.starts_with('-') {
        // Relative time
        let (multi, count) = match &s.chars().last().unwrap() {
            's' => (1, 1),
            'h' => (3600, 1),
            'd' => (3600 * 24, 1),
            'w' => (3600 * 24 * 7, 1),
            'y' => (3600 * 24 * 365, 1),
            'n' if s.ends_with("min") => (60, 3),
            'n' if s.ends_with("mon") => (3600 * 24 * 30, 3),
            _ => return Err(ParseError::Time),
        };

        let s2 = &s[1..s.len() - count];
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32;

        let v = now - s2.parse::<u32>()? * multi;
        Ok(v)
    } else {
        // Absolute time
        match s.as_str() {
            "now" => Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32),
            "yesterday" => {
                Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32 - 3600 * 24)
            }
            "" => Err(ParseError::EmptyString),
            // Unix timestamp parse as default
            _ => {
                // Unix timestamp
                Ok(s.parse::<u32>()?)
            }
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderMetric(String);

#[derive(Debug, PartialEq)]
pub struct RenderPath(PathBuf);

fn path(dir: &Path, metric: &str) -> Result<PathBuf, Error> {
    let path = metric
        .split('.')
        .fold(PathBuf::new(), |acc, x| acc.join(x))
        .with_extension("wsp");
    let full_path = dir.canonicalize()?.join(&path);
    Ok(full_path)
}

fn walk(dir: &Path, metric: &str, q: &RenderQuery) -> Result<Vec<RenderPoint>, Error> {
    let full_path = path(dir, metric)?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32;
    let interval = Interval::new(q.from, q.until).map_err(err_msg)?;

    let archive = WhisperFile::open(&full_path)?.fetch_auto_points(interval, now)?;

    let mut points = Vec::new();

    for (index, value) in archive.values.iter().enumerate() {
        let time = archive.from_interval + archive.step * index as u32;
        points.push(RenderPoint(*value, time));
    }

    Ok(points)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderPoint(Option<f64>, u32);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderResponceEntry {
    target: String,
    datapoints: Vec<RenderPoint>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderResponce {
    entries: Vec<Option<RenderResponceEntry>>,
}

pub fn render_handler(state: State<Args>, params: RenderQuery) -> Result<HttpResponse, Error> {
    let dir = &state.path;
    let response: Vec<RenderResponceEntry> = params
        .target
        .iter()
        .map(|x| RenderResponceEntry {
            target: x.to_string(),
            datapoints: walk(&dir, x, &params).unwrap(),
        })
        .collect();
    Ok(HttpResponse::Ok().json(response))
}

#[cfg(test)]
mod tests {
    use failure::Error;
    use serde_json::to_string;

    use super::*;

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
    fn url_deserialize_time_yesterday_now() -> Result<(), Error> {
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
    fn url_deserialize_time_relative() -> Result<(), Error> {
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
    #[allow(clippy::unreadable_literal)]
    fn render_response_json() {
        let rd = to_string(
            &[RenderResponceEntry {
                target: "entries".into(),
                datapoints: [
                    RenderPoint(Some(1.0), 1311836008),
                    RenderPoint(Some(2.0), 1311836009),
                    RenderPoint(Some(3.0), 1311836010),
                    RenderPoint(Some(5.0), 1311836011),
                    RenderPoint(Some(6.0), 1311836012),
                    RenderPoint(None, 1311836013),
                ]
                .to_vec(),
            }]
            .to_vec(),
        )
        .unwrap();

        let rs =
            r#"[{"target":"entries","datapoints":[[1.0,1311836008],[2.0,1311836009],[3.0,1311836010],[5.0,1311836011],[6.0,1311836012],[null,1311836013]]}]"#;

        assert_eq!(rd, rs);
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
}
