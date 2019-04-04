use actix_web::{Form, HttpResponse, Json, Query, State};
use failure::*;
use serde::*;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use whisper::interval::Interval;
use whisper::WhisperFile;

use crate::opts::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderQuery {
    target: Vec<String>,
    format: RenderFormat,
    from: u32,
    until: u32,
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

fn render_any(args: &Args, params: &RenderQuery) -> Result<HttpResponse, Error> {
    let dir = &args.path;
    let response: Vec<RenderResponceEntry> = params
        .target
        .iter()
        .map(|x| RenderResponceEntry {
            target: x.to_string(),
            datapoints: walk(&dir, x, params).unwrap(),
        })
        .collect();
    Ok(HttpResponse::Ok().json(response))
}

#[allow(clippy::needless_pass_by_value)]
pub fn render_get(state: State<Args>, params: Query<RenderQuery>) -> Result<HttpResponse, Error> {
    render_any(&state, &params.into_inner())
}

#[allow(clippy::needless_pass_by_value)]
pub fn render_form(state: State<Args>, params: Form<RenderQuery>) -> Result<HttpResponse, Error> {
    render_any(&state, &params.into_inner())
}

#[allow(clippy::needless_pass_by_value)]
pub fn render_json(state: State<Args>, params: Json<RenderQuery>) -> Result<HttpResponse, Error> {
    render_any(&state, &params.into_inner())
}

#[cfg(test)]
mod tests {
    use failure::Error;
    use serde_json::to_string;
    use serde_urlencoded;

    use super::*;

    #[test]
    fn url_serialize_one() -> Result<(), Error> {
        let params = RenderQuery {
            format: RenderFormat::Json,
            target: ["app.numUsers".to_owned()].to_vec(),
            from: 0,
            until: 10,
        };

        assert_eq!(
            serde_urlencoded::from_str::<RenderQuery>(
                "target=app.numUsers&format=json&from=0&until=10"
            )?,
            params
        );

        Ok(())
    }

    #[test]
    fn url_serialize_multiple() -> Result<(), Error> {
        let params = RenderQuery {
            format: RenderFormat::Json,
            target: ["app.numUsers".to_owned(), "app.numServers".to_owned()].to_vec(),
            from: 0,
            until: 10,
        };

        assert_eq!(
            serde_urlencoded::from_str::<RenderQuery>(
                "target=app.numUsers&target=app.numServers&format=json&from=0&until=10"
            )?,
            params
        );

        Ok(())
    }

    #[test]
    fn url_serialize_none() -> Result<(), Error> {
        let params = RenderQuery {
            format: RenderFormat::Json,
            target: Vec::new(),
            from: 0,
            until: 10,
        };

        assert_eq!(
            serde_urlencoded::from_str::<RenderQuery>("format=json&from=0&until=10")?,
            params
        );

        Ok(())
    }

    #[test]
    fn response() {
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
}
