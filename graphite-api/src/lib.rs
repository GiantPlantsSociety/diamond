#[macro_use]
extern crate structopt;
#[macro_use]
extern crate serde_derive;
extern crate actix;
extern crate actix_web;
extern crate failure;
extern crate serde;
extern crate serde_json;
extern crate tempfile;
extern crate walkdir;

use actix_web::middleware::Logger;
use actix_web::{http, App, HttpRequest, HttpResponse, Query};
use failure::{err_msg, Error};
use std::path::PathBuf;
use std::str::FromStr;
use walkdir::WalkDir;

pub mod opts;

use opts::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct MetricResponse {
    metrics: Vec<MetricResponseLeaf>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct MetricResponseLeaf {
    name: String,
    path: String,
    is_leaf: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FindFormat {
    TreeJson,
    Completer,
}

impl Default for FindFormat {
    fn default() -> FindFormat {
        FindFormat::TreeJson
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FindQuery {
    query: String,
    #[serde(default)]
    format: FindFormat,
    #[serde(default)]
    wildcards: u8,
    from: u32,
    until: u32,
}

#[derive(Debug, PartialEq)]
pub struct FindPath {
    path: PathBuf,
    pattern: String,
}

impl FromStr for FindPath {
    type Err = Error;

    fn from_str(s: &str) -> Result<FindPath, Error> {
        let mut segments: Vec<&str> = s.split('.').collect();

        match segments.len() {
            len if len > 1 => {
                let pattern = segments.pop().unwrap().to_owned();
                let path = segments
                    .iter()
                    .fold(PathBuf::new(), |acc, x| acc.join(x));

                Ok(FindPath { path, pattern })
            }
            0 => Ok(FindPath {
                path: PathBuf::from(s),
                pattern: "".to_owned(),
            }),
            _ => Err(err_msg("Error")),
        }
    }
}

fn walk_tree(dir: PathBuf, pattern: &str) -> Result<MetricResponse, Error> {
    let mut metrics: Vec<MetricResponseLeaf> = WalkDir::new(dir)
        .min_depth(1)
        .max_depth(1)
        .into_iter()
        .map(|entry| match entry {
            Ok(ref entry) if entry.file_type().is_dir() => {
                let path = format!("{}", entry.path().file_stem().unwrap().to_string_lossy());
                let name = format!("{}", entry.path().file_stem().unwrap().to_string_lossy());
                Some(MetricResponseLeaf {
                    name,
                    path,
                    is_leaf: false,
                })
            }
            Ok(ref entry) if entry.file_type().is_file() => {
                let path = format!("{}", entry.path().file_stem().unwrap().to_string_lossy());
                let name = format!("{}", entry.path().file_stem().unwrap().to_string_lossy());
                Some(MetricResponseLeaf {
                    name,
                    path,
                    is_leaf: true,
                })
            }
            _ => None,
        })
        .filter_map(|x| x)
        .collect();

    metrics.sort_by_key(|k| k.name.clone());
    Ok(MetricResponse { metrics })
}

fn metrics_find(req: HttpRequest<Args>, params: Query<FindQuery>) -> Result<HttpResponse, Error> {
    let args = req.state();
    let dir = &args.path;

    // initial implementation: query should be converted into directory + pattern
    // pattern should be sent to walk_tree
    let path: FindPath = params.query.parse()?;
    let full_path = dir.join(&path.path);

    let metrics = walk_tree(full_path, &path.pattern)?;
    Ok(HttpResponse::Ok().json(metrics))
}

pub fn create_app(opt: Args) -> App<Args> {
    App::with_state(opt)
        .middleware(Logger::default())
        .resource("/metrics/find", |r| {
            r.method(http::Method::GET).with2(metrics_find)
        })
}

#[cfg(test)]
mod tests {
    extern crate serde_urlencoded;

    use super::*;
    use std::fs::create_dir;
    use std::fs::File;
    use std::path::{Path, PathBuf};
    use tempfile::{Builder, TempDir};

    fn get_temp_dir() -> TempDir {
        Builder::new()
            .prefix("carbon")
            .tempdir()
            .expect("Temp dir created")
    }

    #[test]
    fn test_walk_tree() -> Result<(), Error> {
        let dir = get_temp_dir();
        let path = dir.path();
        let path1 = path.join(Path::new("foo"));
        let path2 = path.join(Path::new("bar"));
        let path3 = path.join(Path::new("foobar.wsp"));

        create_dir(&path1)?;
        create_dir(&path2)?;

        let _file = File::create(&path3).unwrap();

        let metric = walk_tree(path.to_path_buf(), "*");

        let mut metric_cmp = vec![
            MetricResponseLeaf {
                name: "foo".into(),
                path: "foo".into(),
                is_leaf: false,
            },
            MetricResponseLeaf {
                name: "bar".into(),
                path: "bar".into(),
                is_leaf: false,
            },
            MetricResponseLeaf {
                name: "foobar".into(),
                path: "foobar".into(),
                is_leaf: true,
            },
        ];

        metric_cmp.sort_by_key(|k| k.name.clone());

        assert_eq!(
            metric.unwrap(),
            MetricResponse {
                metrics: metric_cmp
            }
        );

        Ok(())
    }

    #[test]
    fn test_serialize() -> Result<(), Error> {
        let params = FindQuery {
            query: "123".to_owned(),
            format: FindFormat::TreeJson,
            wildcards: 1,
            from: 0,
            until: 10,
        };

        assert_eq!(
            "query=123&format=treejson&wildcards=1&from=0&until=10",
            serde_urlencoded::to_string(params.clone())?
        );

        let params2 = FindQuery {
            query: "123".to_owned(),
            format: FindFormat::Completer,
            wildcards: 0,
            from: 0,
            until: 10,
        };

        assert_eq!(
            "query=123&format=completer&wildcards=0&from=0&until=10",
            serde_urlencoded::to_string(params2.clone())?
        );

        Ok(())
    }

    #[test]
    fn test_deserialize() -> Result<(), Error> {
        let params = FindQuery {
            query: "123".to_owned(),
            format: FindFormat::TreeJson,
            wildcards: 1,
            from: 0,
            until: 10,
        };

        assert_eq!(
            serde_urlencoded::from_str("query=123&format=treejson&wildcards=1&from=0&until=10"),
            Ok(params)
        );

        let params2 = FindQuery {
            query: "123".to_owned(),
            format: FindFormat::Completer,
            wildcards: 0,
            from: 0,
            until: 10,
        };

        assert_eq!(
            serde_urlencoded::from_str("query=123&format=completer&wildcards=0&from=0&until=10"),
            Ok(params2)
        );

        Ok(())
    }

    #[test]
    fn test_deserialize_default() -> Result<(), Error> {
        let params = FindQuery {
            query: "123".to_owned(),
            format: FindFormat::default(),
            wildcards: u8::default(),
            from: 0,
            until: 10,
        };

        assert_eq!(
            serde_urlencoded::from_str("query=123&from=0&until=10"),
            Ok(params)
        );

        Ok(())
    }

    #[test]
    fn findpath_convert() -> Result<(), Error> {
        let path = FindPath {
            path: PathBuf::from("123/456/"),
            pattern: "789".to_owned(),
        };

        assert_eq!(
            "123.456.789".parse().ok(),
            Some(path)
        );

        Ok(())
    }
}
