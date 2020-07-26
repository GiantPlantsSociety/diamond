use crate::render_target::*;

#[derive(Copy, Clone, Debug)]
struct Point(pub i32, pub f64);

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        let min_value = self.1 / 1000000_f64;
        (self.0 == other.0) && ((self.1 - other.1).abs() < min_value)
    }
}

#[derive(Clone, PartialEq, Debug)]
struct Series {
    pub name: String,
    pub points: Vec<Point>,
}

trait Storage {
    fn fetch(&self, path: PathExpression) -> Vec<Series>;
}

trait ExpressionExec {
    fn execute(&self, expr: Expression) -> Vec<Series>;
    fn args(args: Vec<Arg>) -> (Vec<Expression>, Vec<LiteralValue>);
    fn resolve_series(&self, series: Vec<Expression>) -> Vec<Series>;
}

impl<T: Storage> ExpressionExec for T {
    fn execute(&self, expr: Expression) -> Vec<Series> {
        match expr {
            Expression::Path(path_expression) => self.fetch(path_expression),
            Expression::Call(Call {
                function,
                args,
                named_args,
            }) => {
                let (series, literals) = Self::args(args);
                match (
                    function.as_str(),
                    series,
                    literals.as_slice(),
                    named_args.as_slice(),
                ) {
                    ("sumSeries", series, [], []) => {
                        let series_values = self.resolve_series(series);
                        sum_series(series_values, "".to_owned())
                    }
                    ("absolute", series, [], []) => {
                        let series_values = self.resolve_series(series);
                        absolute(series_values)
                    }
                    // waiting for feature(move_ref_pattern)
                    ("alias", ref _series, [LiteralValue::String(_name)], []) => unimplemented!(),
                    ("aliasByMetric", _series, [], []) => unimplemented!(),
                    ("aliasByNode", _series, [], []) => unimplemented!(),
                    ("averageSeries", series, [], []) => {
                        let series_values = self.resolve_series(series);
                        average_series(series_values, "".to_owned())
                    }
                    ("countSeries", series, [], []) => {
                        let series_values = self.resolve_series(series);
                        count_series(series_values, "".to_owned())
                    }
                    ("divideSeries", series, [], []) => {
                        if series.len() != 2 {
                            Vec::new()
                        } else {
                            let series_values = self.resolve_series(series);
                            divide_series(series_values, "".to_owned())
                        }
                    }
                    ("diffSeries", series, [], []) => {
                        if series.len() != 2 {
                            Vec::new()
                        } else {
                            let series_values = self.resolve_series(series);
                            diff_series(series_values, "".to_owned())
                        }
                    }
                    ("maxSeries", series, [], []) => {
                        let series_values = self.resolve_series(series);
                        max_series(series_values, "".to_owned())
                    }
                    ("minSeries", series, [], []) => {
                        let series_values = self.resolve_series(series);
                        min_series(series_values, "".to_owned())
                    }
                    ("multiplySeries", series, [], []) => {
                        let series_values = self.resolve_series(series);
                        multiply_series(series_values, "".to_owned())
                    }
                    _ => unimplemented!(),
                }
            }
            _ => unimplemented!(),
        }
    }

    fn args(args: Vec<Arg>) -> (Vec<Expression>, Vec<LiteralValue>) {
        let mut values = Vec::new();
        let mut literals = Vec::new();

        for arg in args {
            match arg {
                Arg::Expression(expression) => values.push(expression),
                Arg::Literal(literal) => literals.push(literal),
            };
        }
        (values, literals)
    }

    fn resolve_series(&self, series: Vec<Expression>) -> Vec<Series> {
        series
            .into_iter()
            .map(|x| self.execute(x))
            .flatten()
            .collect::<Vec<_>>()
    }
}

fn sum_series(series: Vec<Series>, name: String) -> Vec<Series> {
    if series.len() == 0 {
        return series;
    }

    let init = series
        .iter()
        .next()
        .unwrap()
        .points
        .iter()
        .map(|Point(time, _)| Point(*time, 0_f64))
        .collect();

    let sum = Series {
        name,
        points: series.into_iter().fold(init, |acc: Vec<Point>, serie| {
            acc.into_iter()
                .zip(serie.points.into_iter())
                .map(|(Point(time, x), Point(_, y))| Point(time, x + y))
                .collect::<Vec<_>>()
        }),
    };
    vec![sum]
}

fn absolute(series: Vec<Series>) -> Vec<Series> {
    series
        .into_iter()
        .map(|serie| Series {
            name: serie.name,
            points: serie
                .points
                .into_iter()
                .map(|Point(time, x)| Point(time, x.abs()))
                .collect::<Vec<_>>(),
        })
        .collect::<Vec<_>>()
}

fn count_series(series: Vec<Series>, name: String) -> Vec<Series> {
    let count = f64::from(series.len() as i32);

    match series.into_iter().nth(0) {
        None => Vec::new(),
        Some(first) => {
            let count_series = Series {
                name,
                points: first
                    .points
                    .into_iter()
                    .map(|Point(time, _)| Point(time, count))
                    .collect(),
            };
            vec![count_series]
        }
    }
}

fn average_series(series: Vec<Series>, name: String) -> Vec<Series> {
    let count = f64::from(series.len() as i32);

    if series.len() == 0 {
        return series;
    }

    let init = series
        .iter()
        .next()
        .unwrap()
        .points
        .iter()
        .map(|Point(time, _)| Point(*time, 0_f64))
        .collect();

    let avg = Series {
        name,
        points: series
            .into_iter()
            .fold(init, |acc: Vec<Point>, serie| {
                acc.into_iter()
                    .zip(serie.points.into_iter())
                    .map(|(Point(time, x), Point(_, y))| Point(time, x + y))
                    .collect::<Vec<_>>()
            })
            .into_iter()
            .map(|Point(time, x)| Point(time, x / count))
            .collect::<Vec<_>>(),
    };

    vec![avg]
}

fn divide_series(series: Vec<Series>, name: String) -> Vec<Series> {
    if let [left, right] = series.as_slice() {
        let divide = Series {
            name,
            points: left
                .points
                .iter()
                .zip(right.points.iter())
                .map(|(Point(time, x), Point(_, y))| Point(*time, x / y))
                .collect::<Vec<_>>(),
        };
        vec![divide]
    } else {
        Vec::new()
    }
}

fn diff_series(series: Vec<Series>, name: String) -> Vec<Series> {
    if let [left, right] = series.as_slice() {
        let divide = Series {
            name,
            points: left
                .points
                .iter()
                .zip(right.points.iter())
                .map(|(Point(time, x), Point(_, y))| Point(*time, x - y))
                .collect::<Vec<_>>(),
        };
        vec![divide]
    } else {
        Vec::new()
    }
}

fn max_series(series: Vec<Series>, name: String) -> Vec<Series> {
    let init = series
        .iter()
        .next()
        .unwrap()
        .points
        .iter()
        .map(|Point(time, _)| Point(*time, 0_f64))
        .collect();

    let max_series = Series {
        name,
        points: series.into_iter().fold(init, |acc: Vec<Point>, serie| {
            acc.into_iter()
                .zip(serie.points.into_iter())
                .map(|(Point(time, x), Point(_, y))| Point(time, x.max(y)))
                .collect::<Vec<_>>()
        }),
    };
    vec![max_series]
}

fn min_series(series: Vec<Series>, name: String) -> Vec<Series> {
    let init = series
        .iter()
        .next()
        .unwrap()
        .points
        .iter()
        .map(|Point(time, _)| Point(*time, 0_f64))
        .collect();

    let min_series = Series {
        name,
        points: series.into_iter().fold(init, |acc: Vec<Point>, serie| {
            acc.into_iter()
                .zip(serie.points.into_iter())
                .map(|(Point(time, x), Point(_, y))| Point(time, x.min(y)))
                .collect::<Vec<_>>()
        }),
    };
    vec![min_series]
}

fn multiply_series(series: Vec<Series>, name: String) -> Vec<Series> {
    let init = series
        .iter()
        .next()
        .unwrap()
        .points
        .iter()
        .map(|Point(time, _)| Point(*time, 1_f64))
        .collect();

    let mul = Series {
        name,
        points: series.into_iter().fold(init, |acc: Vec<Point>, serie| {
            acc.into_iter()
                .zip(serie.points.into_iter())
                .map(|(Point(time, x), Point(_, y))| Point(time, x * y))
                .collect::<Vec<_>>()
        }),
    };
    vec![mul]
}

#[derive(Clone)]
pub struct StorageConst(Vec<Series>);

impl Storage for StorageConst {
    fn fetch(&self, _path: PathExpression) -> Vec<Series> {
        self.0.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expression_path_test() {
        let s = StorageConst(vec![Series {
            name: "path.to.metric".to_owned(),
            points: vec![Point(1, 0.1)],
        }]);
        let a: Expression = "path.to.metric".parse().unwrap();
        assert_eq!(
            s.execute(a),
            vec![Series {
                name: "path.to.metric".to_owned(),
                points: vec![Point(1, 0.1)]
            }]
        );
    }

    #[test]
    fn test_execution() {
        let s = StorageConst(vec![
            Series {
                name: "1".to_owned(),
                points: vec![Point(1, 0.1), Point(2, 0.2)],
            },
            Series {
                name: "2".to_owned(),
                points: vec![Point(1, 0.2), Point(2, 0.4)],
            },
            Series {
                name: "3".to_owned(),
                points: vec![Point(1, 0.3), Point(2, 0.6)],
            },
        ]);
        let a: Expression = "sumSeries(path.to.metric)".parse().unwrap();
        assert_eq!(
            s.execute(a),
            vec![Series {
                name: "".to_owned(),
                points: vec![Point(1, 0.6), Point(2, 1.2)]
            }]
        );
    }

    #[test]
    fn test_execution2() {
        let s = StorageConst(vec![
            Series {
                name: "".to_owned(),
                points: vec![Point(1, 1_f64), Point(2, 1_f64)],
            },
            Series {
                name: "".to_owned(),
                points: vec![Point(1, 2_f64), Point(2, 0.5)],
            },
            Series {
                name: "".to_owned(),
                points: vec![Point(1, 3_f64), Point(2, 4_f64)],
            },
        ]);
        let a: Expression = "multiplySeries(path.to.metric)".parse().unwrap();
        assert_eq!(
            s.execute(a),
            vec![Series {
                name: "".to_owned(),
                points: vec![Point(1, 6_f64), Point(2, 2_f64)]
            }]
        );
    }
}
