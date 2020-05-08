use glob::Pattern;
use serde::*;
use std::ffi::OsStr;
use std::fs;
use std::iter::successors;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use whisper::interval::Interval;
use whisper::{ArchiveData, WhisperFile};

use crate::error::ResponseError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderPoint(pub Option<f64>, pub u32);

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct MetricResponseLeaf {
    pub name: String,
    pub path: String,
    pub is_leaf: bool,
}

impl MetricResponseLeaf {
    fn new(name: &str, dir: &str, is_leaf: bool) -> Self {
        let path = if !dir.is_empty() {
            format!("{}.{}", dir, name)
        } else {
            name.to_owned()
        };
        MetricResponseLeaf {
            name: name.to_owned(),
            path,
            is_leaf,
        }
    }
}

pub trait Walker {
    fn walk(&self, metric: &str, interval: Interval) -> Result<Vec<RenderPoint>, ResponseError>;
    fn walk_tree(
        &self,
        subdir: &Path,
        pattern: &Pattern,
    ) -> Result<Vec<MetricResponseLeaf>, ResponseError>;
}

#[derive(Clone)]
pub struct WalkerPath(pub PathBuf);

impl Walker for WalkerPath {
    fn walk(&self, metric: &str, interval: Interval) -> Result<Vec<RenderPoint>, ResponseError> {
        let full_path = path(self.0.as_path(), metric)?;
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as u32;
        let ArchiveData {
            from_interval,
            step,
            values,
            ..
        } = WhisperFile::open(&full_path)?.fetch_auto_points(interval, now)?;
        let timestamps = successors(Some(from_interval), |i| i.checked_add(step));
        let points = values
            .into_iter()
            .zip(timestamps)
            .map(|(value, time)| RenderPoint(value, time))
            .collect();

        Ok(points)
    }

    fn walk_tree(
        &self,
        subdir: &Path,
        pattern: &Pattern,
    ) -> Result<Vec<MetricResponseLeaf>, ResponseError> {
        let full_path = self.0.canonicalize()?.join(&subdir);
        let dir_metric = subdir
            .components()
            .filter_map(|x| x.as_os_str().to_str())
            .fold(String::new(), |acc, x| acc + x);

        let mut metrics: Vec<MetricResponseLeaf> = fs::read_dir(&full_path)?
            .filter_map(|entry| {
                let rentry = entry.ok()?;
                let path = rentry
                    .path()
                    .strip_prefix(&full_path)
                    .map(std::borrow::ToOwned::to_owned)
                    .ok()?;
                let file_type = rentry.file_type().ok()?;

                if pattern.matches_path(&path) {
                    if file_type.is_dir() {
                        let name = path.file_name()?.to_str()?.to_owned();
                        Some(MetricResponseLeaf::new(&name, &dir_metric, false))
                    } else if path.extension() == Some(OsStr::new("wsp")) {
                        let name = path.file_stem()?.to_str()?.to_owned();
                        Some(MetricResponseLeaf::new(&name, &dir_metric, true))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        metrics.sort_by_key(|k| k.name.clone());
        Ok(metrics)
    }
}

#[inline]
fn path(dir: &Path, metric: &str) -> Result<PathBuf, ResponseError> {
    let path = metric
        .split('.')
        .fold(PathBuf::new(), |acc, x| acc.join(x))
        .with_extension("wsp");
    let full_path = dir
        .canonicalize()
        .map_err(|_| ResponseError::Path)?
        .join(&path);
    Ok(full_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::create_dir;
    use std::fs::File;
    use std::path::Path;

    fn get_temp_dir() -> tempfile::TempDir {
        tempfile::Builder::new()
            .prefix("diamond-api")
            .tempdir()
            .expect("Temp dir created")
    }

    #[test]
    fn walk_tree_verify() -> Result<(), Box<dyn std::error::Error>> {
        let dir = get_temp_dir();
        let path = dir.path();
        let path1 = path.join(Path::new("foo"));
        let path2 = path.join(Path::new("bar"));
        let path3 = path.join(Path::new("foobar.wsp"));
        let path4 = path1.join(Path::new("bar.wsp"));

        create_dir(&path1)?;
        create_dir(&path2)?;
        let _file1 = File::create(&path3)?;
        let _file2 = File::create(&path4)?;

        let metric = WalkerPath(path.to_owned()).walk_tree(&Path::new(""), &Pattern::new("*")?)?;

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
        assert_eq!(metric, metric_cmp);

        let metric2 =
            WalkerPath(path.to_owned()).walk_tree(&Path::new("foo"), &Pattern::new("*")?)?;

        let mut metric_cmp2 = vec![MetricResponseLeaf {
            name: "bar".into(),
            path: "foo.bar".into(),
            is_leaf: true,
        }];

        metric_cmp2.sort_by_key(|k| k.name.clone());
        assert_eq!(metric2, metric_cmp2);

        Ok(())
    }
}
