use crate::render_target::*;
use std::error::Error;
use strum_macros::EnumString;

mod functions;
use functions::*;

#[derive(Copy, Clone, Debug)]
pub struct Point(pub i32, pub f64);

impl PartialEq for Point {
    fn eq(&self, other: &Self) -> bool {
        let min_value = self.1 / 1000000_f64;
        (self.0 == other.0) && ((self.1 - other.1).abs() < min_value)
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Series {
    pub name: String,
    pub points: Vec<Point>,
}

#[derive(EnumString)]
#[strum(serialize_all = "camelCase")]
pub enum DFunction {
    SumSeries,
    Absolute,
    Alias,
    AliasByMetric,
    AliasByNode,
    AverageSeries,
    CountSeries,
    DivideSeries,
    DiffSeries,
    MaxSeries,
    MinSeries,
    MultiplySeries,
}

trait Storage {
    fn fetch(&self, path: PathExpression) -> Vec<Series>;
}

trait ExpressionExec {
    fn apply(&self, expr: Expression) -> Result<Vec<Series>, Box<dyn Error>>;
    fn args(args: Vec<Arg>) -> (Vec<Expression>, Vec<LiteralValue>);
    fn resolve_series(&self, series: Vec<Expression>) -> Vec<Series>;
    fn apply_function(
        &self,
        function: DFunction,
        series: Vec<Expression>,
        literals: &[LiteralValue],
        named_args: &[(String, Arg)],
    ) -> Result<Vec<Series>, Box<dyn Error>>;
}

impl<T: Storage> ExpressionExec for T {
    fn apply(&self, expr: Expression) -> Result<Vec<Series>, Box<dyn Error>> {
        let res = match expr {
            Expression::Path(path_expression) => self.fetch(path_expression),
            Expression::Call(Call {
                function,
                args,
                named_args,
            }) => {
                let function: DFunction = function.parse()?;
                let (series, literals) = Self::args(args);
                self.apply_function(function, series, &literals, &named_args)?
            }
            _ => unimplemented!(),
        };
        Ok(res)
    }

    #[inline]
    fn apply_function(
        &self,
        function: DFunction,
        series: Vec<Expression>,
        literals: &[LiteralValue],
        named_args: &[(String, Arg)],
    ) -> Result<Vec<Series>, Box<dyn Error>> {
        let res = match (function, series, literals, named_args) {
            (DFunction::SumSeries, series, [], []) => {
                let series_values = self.resolve_series(series);
                sum_series(series_values, "".to_owned())
                    .into_iter()
                    .collect()
            }
            (DFunction::Absolute, series, [], []) => {
                let series_values = self.resolve_series(series);
                absolute(series_values)
            }
            // waiting for feature(move_ref_pattern)
            (DFunction::Alias, series, literals, []) => {
                if let Some(LiteralValue::String(name)) = literals.into_iter().next() {
                    let series_values = self.resolve_series(series);
                    alias(series_values, name.to_owned())
                } else {
                    Vec::new()
                }
            }
            (DFunction::AliasByMetric, series, [], []) => {
                let series_values = self.resolve_series(series);
                alias_by_metric(series_values)
            }
            (DFunction::AliasByNode, series, [], []) => {
                let series_values = self.resolve_series(series);
                let nodes = resolve_numbers(literals);
                alias_by_node(series_values, nodes)
            }
            (DFunction::AverageSeries, series, [], []) => {
                let series_values = self.resolve_series(series);
                average_series(series_values, "".to_owned())
            }
            (DFunction::CountSeries, series, [], []) => {
                let series_values = self.resolve_series(series);
                count_series(series_values, "".to_owned())
            }
            (DFunction::DivideSeries, series, [], []) => {
                if series.len() != 2 {
                    Vec::new()
                } else {
                    let series_values = self.resolve_series(series);
                    divide_series(series_values, "".to_owned())
                }
            }
            (DFunction::DiffSeries, series, [], []) => {
                if series.len() != 2 {
                    Vec::new()
                } else {
                    let series_values = self.resolve_series(series);
                    diff_series(series_values, "".to_owned())
                }
            }
            (DFunction::MaxSeries, series, [], []) => {
                let series_values = self.resolve_series(series);
                max_series(series_values, "".to_owned())
            }
            (DFunction::MinSeries, series, [], []) => {
                let series_values = self.resolve_series(series);
                min_series(series_values, "".to_owned())
            }
            (DFunction::MultiplySeries, series, [], []) => {
                let series_values = self.resolve_series(series);
                multiply_series(series_values, "".to_owned())
            }
            _ => unimplemented!(),
        };
        Ok(res)
    }

    #[inline]
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
            .map(|x| self.apply(x).unwrap())
            .flatten()
            .collect::<Vec<_>>()
    }
}

fn resolve_numbers(values: &[LiteralValue]) -> Vec<i64> {
    values
        .iter()
        .flat_map(|x| {
            if let &LiteralValue::Integer(value) = x {
                Some(value)
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
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
            s.apply(a).unwrap(),
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
            s.apply(a).unwrap(),
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
            s.apply(a).unwrap(),
            vec![Series {
                name: "".to_owned(),
                points: vec![Point(1, 6_f64), Point(2, 2_f64)]
            }]
        );
    }
}
