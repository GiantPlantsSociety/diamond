use actix_web::{Form, HttpRequest, HttpResponse, Json, Query};
use failure::Error;
use glob::Pattern;
use std::convert::From;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use opts::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct MetricResponse {
    metrics: Vec<MetricResponseLeaf>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
struct MetricResponseLeaf {
    name: String,
    path: String,
    is_leaf: bool,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
struct JsonTreeLeaf {
    text: String,
    id: String,
    #[serde(rename = "allowChildren")]
    allow_children: u8,
    expandable: u8,
    leaf: u8,
}

impl From<MetricResponseLeaf> for JsonTreeLeaf {
    fn from(m: MetricResponseLeaf) -> JsonTreeLeaf {
        if m.is_leaf {
            JsonTreeLeaf {
                text: m.path,
                id: m.name,
                allow_children: 0,
                expandable: 0,
                leaf: 1,
            }
        } else {
            JsonTreeLeaf {
                text: m.path,
                id: m.name,
                allow_children: 1,
                expandable: 1,
                leaf: 0,
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
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
    pattern: Pattern,
}

impl FindPath {
    fn from(query: FindQuery) -> Result<FindPath, Error> {
        let s = &query.query;
        let mut segments: Vec<&str> = s.split('.').collect();

        let (path, pattern) = match segments.len() {
            len if len > 1 => {
                let pattern = segments.pop().ok_or(FindError::UnknownParse)?;
                let path = segments.iter().fold(PathBuf::new(), |acc, x| acc.join(x));
                (path, pattern)
            }
            1 => (PathBuf::from(""), s.as_str()),
            _ => return Err(FindError::Unknown.into()),
        };

        let pattern_str = match query.wildcards {
            0 => pattern.to_owned(),
            1 => [&pattern, "*"].concat(),
            _ => pattern.to_owned(),
        };

        Ok(FindPath {
            path,
            pattern: Pattern::new(&pattern_str)?,
        })
    }
}

#[derive(Fail, Debug)]
pub enum FindError {
    #[fail(display = "Unknown error")]
    Unknown,
    #[fail(display = "Unknown parse error")]
    UnknownParse,
}

fn create_leaf(name: &str, dir: &str, is_leaf: bool) -> MetricResponseLeaf {
    let path = if !dir.is_empty() {
        format!("{}.{}", dir, name)
    } else {
        name.clone().to_owned()
    };

    MetricResponseLeaf {
        name: name.to_owned(),
        path,
        is_leaf,
    }
}

fn walk_tree(
    dir: &Path,
    subdir: &Path,
    pattern: &Pattern,
) -> Result<Vec<MetricResponseLeaf>, Error> {
    let full_path = dir.canonicalize()?.join(&subdir);
    let dir_metric = subdir
        .components()
        .map(|x| x.as_os_str().to_str().unwrap())
        .collect::<Vec<&str>>()
        .concat();

    let mut metrics: Vec<MetricResponseLeaf> = fs::read_dir(&full_path)?
        .into_iter()
        .map(|entry| match entry {
            Ok(ref entry)
                if pattern.matches_path(&entry.path().strip_prefix(&full_path).unwrap())
                    && entry.file_type().unwrap().is_dir() =>
            {
                let name = entry
                    .path()
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned();
                Some(create_leaf(&name, &dir_metric, false))
            }
            Ok(ref entry)
                if pattern.matches_path(&entry.path().strip_prefix(&full_path).unwrap())
                    && entry.file_type().unwrap().is_file()
                    && entry.path().extension().unwrap() == OsStr::new("wsp") =>
            {
                let name = entry
                    .path()
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_owned();
                Some(create_leaf(&name, &dir_metric, true))
            }
            _ => None,
        }).filter_map(|x| x)
        .collect();

    metrics.sort_by_key(|k| k.name.clone());
    Ok(metrics)
}

fn metrics_find(args: &Args, params: FindQuery) -> Result<HttpResponse, Error> {
    let dir = &args.path;
    let path: FindPath = FindPath::from(params.clone())?;

    let metrics = walk_tree(&dir, &path.path, &path.pattern)?;

    if params.format == FindFormat::TreeJson {
        let metrics_json: Vec<JsonTreeLeaf> = metrics
            .iter()
            .map(|x| JsonTreeLeaf::from(x.to_owned()))
            .collect();
        Ok(HttpResponse::Ok().json(metrics_json))
    } else {
        let metrics_completer = MetricResponse { metrics };
        Ok(HttpResponse::Ok().json(metrics_completer))
    }
}

pub fn metrics_find_get(
    req: HttpRequest<Args>,
    params: Query<FindQuery>,
) -> Result<HttpResponse, Error> {
    metrics_find(req.state(), params.into_inner())
}

pub fn metrics_find_form(
    req: HttpRequest<Args>,
    params: Form<FindQuery>,
) -> Result<HttpResponse, Error> {
    metrics_find(req.state(), params.into_inner())
}

pub fn metrics_find_json(
    req: HttpRequest<Args>,
    params: Json<FindQuery>,
) -> Result<HttpResponse, Error> {
    metrics_find(req.state(), params.into_inner())
}

#[cfg(test)]
mod tests {
    extern crate serde_urlencoded;
    extern crate tempfile;

    use super::*;
    use std::fs::create_dir;
    use std::fs::File;
    use std::path::{Path, PathBuf};

    fn get_temp_dir() -> tempfile::TempDir {
        tempfile::Builder::new()
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
        let path4 = path1.join(Path::new("bar.wsp"));

        create_dir(&path1)?;
        create_dir(&path2)?;
        let _file1 = File::create(&path3).unwrap();
        let _file2 = File::create(&path4).unwrap();

        let metric = walk_tree(&path, &Path::new(""), &Pattern::new("*")?);

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

        let metric2 = walk_tree(&path, &Path::new("foo"), &Pattern::new("*")?);

        let mut metric_cmp2 = vec![MetricResponseLeaf {
            name: "bar".into(),
            path: "foo.bar".into(),
            is_leaf: true,
        }];

        metric_cmp2.sort_by_key(|k| k.name.clone());

        assert_eq!(metric2.unwrap().metrics, metric_cmp2);

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
            pattern: Pattern::new("789")?,
        };

        let params = FindQuery {
            query: "123.456.789".to_owned(),
            format: FindFormat::default(),
            wildcards: u8::default(),
            from: 0,
            until: 10,
        };

        assert_eq!(FindPath::from(params)?, path);

        Ok(())
    }

    #[test]
    fn test_query() -> Result<(), Error> {
        let params = FindQuery {
            query: "123".to_owned(),
            format: FindFormat::TreeJson,
            wildcards: 1,
            from: 0,
            until: 10,
        };

        let path = FindPath {
            path: PathBuf::from(""),
            pattern: Pattern::new("123*")?,
        };

        assert_eq!(FindPath::from(params)?, path);

        Ok(())
    }
}
