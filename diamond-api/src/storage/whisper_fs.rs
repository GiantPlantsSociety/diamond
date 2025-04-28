use std::borrow::Cow;
use std::ffi::OsStr;
use std::fs;
use std::iter::successors;
use std::path::{Path, PathBuf};
use whisper::interval::Interval;
use whisper::{ArchiveData, WhisperFile};

use super::storage::*;
use crate::error::ResponseError;
pub use crate::render_target::ast::{PathExpression, PathWord};

#[derive(Clone)]
pub struct WhisperFileSystemStorage(pub PathBuf);

impl Storage for WhisperFileSystemStorage {
    fn find(
        &self,
        path_expression: &PathExpression,
    ) -> Result<Vec<MetricResponseLeaf>, ResponseError> {
        let mut paths = Vec::new();
        walk_tree(
            &self.0,
            &MetricName::default(),
            &path_expression.0,
            &mut paths,
        )?;
        paths.sort_by_cached_key(|k| k.0.clone());

        Ok(paths
            .into_iter()
            .map(|(metric_name, fs_path)| MetricResponseLeaf {
                name: metric_name,
                is_leaf: fs_path.is_file(),
            })
            .collect())
    }

    fn query(
        &self,
        path_expression: &PathExpression,
        interval: Interval,
        now: u64,
    ) -> Result<Vec<StorageResponse>, ResponseError> {
        let mut paths = Vec::new();
        walk_tree(
            &self.0,
            &MetricName::default(),
            &path_expression.0,
            &mut paths,
        )?;

        let mut responses = Vec::new();
        for (metric_name, fs_path) in paths {
            let ArchiveData {
                from_interval,
                step,
                values,
                ..
            } = WhisperFile::open(&fs_path)?.fetch_auto_points(interval, now as u32)?;
            let timestamps = successors(Some(from_interval), |i| i.checked_add(step));
            let points = values
                .into_iter()
                .zip(timestamps)
                .map(|(value, time)| RenderPoint(value, time))
                .collect();

            responses.push(StorageResponse {
                name: metric_name,
                data: points,
            });
        }
        Ok(responses)
    }
}

fn file_name(path: &Path) -> Option<Cow<'_, str>> {
    if path.is_dir() {
        Some(path.file_name()?.to_string_lossy())
    } else if path.is_file() && path.extension() == Some(OsStr::new("wsp")) {
        Some(path.file_stem()?.to_string_lossy())
    } else {
        None
    }
}

fn walk_tree(
    dir: &Path,
    path_prefix: &MetricName,
    path_words: &[PathWord],
    acc: &mut Vec<(MetricName, PathBuf)>,
) -> Result<(), ResponseError> {
    match path_words.len() {
        0 => {}
        1 => {
            let regex = path_words[0].to_regex().map_err(|_| ResponseError::Path)?;
            for entry in fs::read_dir(dir)? {
                let path = entry?.path();
                if let Some(file_name) = file_name(&path) {
                    if regex.is_match(&file_name) {
                        let storage_path = path_prefix.join(file_name);
                        acc.push((storage_path, path));
                    }
                }
            }
        }
        _ => {
            let regex = path_words[0].to_regex().map_err(|_| ResponseError::Path)?;
            for entry in fs::read_dir(dir)? {
                let path = entry?.path();
                if let Some(file_name) = file_name(&path) {
                    if regex.is_match(&file_name) {
                        let storage_path = path_prefix.join(file_name);
                        walk_tree(&path, &storage_path, &path_words[1..], acc)?;
                    }
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::fs::create_dir;
    use std::path::Path;
    use std::str::FromStr;

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

        let metric =
            WhisperFileSystemStorage(path.to_owned()).find(&PathExpression::from_str("*")?)?;

        let mut metric_cmp = vec![
            MetricResponseLeaf {
                name: "foo".parse().unwrap(),
                is_leaf: false,
            },
            MetricResponseLeaf {
                name: "bar".parse().unwrap(),
                is_leaf: false,
            },
            MetricResponseLeaf {
                name: "foobar".parse().unwrap(),
                is_leaf: true,
            },
        ];

        metric_cmp.sort_by_key(|k| k.name.clone());
        assert_eq!(metric, metric_cmp);

        let metric2 =
            WhisperFileSystemStorage(path.to_owned()).find(&PathExpression::from_str("foo.*")?)?;

        let mut metric_cmp2 = vec![MetricResponseLeaf {
            name: "foo.bar".parse().unwrap(),
            is_leaf: true,
        }];

        metric_cmp2.sort_by_key(|k| k.name.clone());
        assert_eq!(metric2, metric_cmp2);

        Ok(())
    }
}
