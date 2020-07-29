use super::{Point, Series};

fn aggregation_series(
    series: Vec<Series>,
    name: String,
    initial: f64,
    f: impl Fn((Point, Point)) -> Point,
) -> Vec<Series> {
    match series.iter().next() {
        Some(first) => {
            let init = first
                .points
                .iter()
                .map(|Point(time, _)| Point(*time, initial))
                .collect();

            let aggr = Series {
                name,
                points: series.into_iter().fold(init, |acc: Vec<Point>, serie| {
                    acc.into_iter()
                        .zip(serie.points.into_iter())
                        .map(&f)
                        .collect::<Vec<_>>()
                }),
            };
            vec![aggr]
        }
        _ => Vec::new(),
    }
}

fn adjust_series(series: Vec<Series>, f: impl Fn(Point) -> Point) -> Vec<Series> {
    series
        .into_iter()
        .map(|serie| Series {
            name: serie.name,
            points: serie.points.into_iter().map(&f).collect::<Vec<_>>(),
        })
        .collect::<Vec<_>>()
}

fn pair_series(
    series: Vec<Series>,
    name: String,
    f: impl Fn((&Point, &Point)) -> Point,
) -> Vec<Series> {
    if let [left, right] = series.as_slice() {
        let divide = Series {
            name,
            points: left
                .points
                .iter()
                .zip(right.points.iter())
                .map(&f)
                .collect::<Vec<_>>(),
        };
        vec![divide]
    } else {
        Vec::new()
    }
}

pub fn sum_series(series: Vec<Series>, name: String) -> Vec<Series> {
    fn sum(pair: (Point, Point)) -> Point {
        let (x, y) = pair;
        Point(x.0, x.1 + y.1)
    };

    aggregation_series(series, name, 0.0, sum)
}

pub fn max_series(series: Vec<Series>, name: String) -> Vec<Series> {
    fn max(pair: (Point, Point)) -> Point {
        let (x, y) = pair;
        Point(x.0, x.1.max(y.1))
    };

    aggregation_series(series, name, 0.0, max)
}

pub fn min_series(series: Vec<Series>, name: String) -> Vec<Series> {
    fn min(pair: (Point, Point)) -> Point {
        let (x, y) = pair;
        Point(x.0, x.1.min(y.1))
    };

    aggregation_series(series, name, 0.0, min)
}

pub fn multiply_series(series: Vec<Series>, name: String) -> Vec<Series> {
    fn multiply(pair: (Point, Point)) -> Point {
        let (x, y) = pair;
        Point(x.0, x.1 * y.1)
    };

    aggregation_series(series, name, 1.0, multiply)
}

pub fn absolute(series: Vec<Series>) -> Vec<Series> {
    adjust_series(series, |Point(time, x)| Point(time, x.abs()))
}

pub fn count_series(series: Vec<Series>, _name: String) -> Vec<Series> {
    let count = f64::from(series.len() as i32);
    adjust_series(series, |Point(time, _)| Point(time, count))
}

pub fn average_series(series: Vec<Series>, name: String) -> Vec<Series> {
    let count = f64::from(series.len() as i32);
    let avg = |x: Point| Point(x.0, x.1 / count);
    adjust_series(sum_series(series, name), avg)
}

pub fn divide_series(series: Vec<Series>, name: String) -> Vec<Series> {
    pair_series(series, name, |(Point(time, x), Point(_, y))| {
        Point(*time, x / y)
    })
}

pub fn diff_series(series: Vec<Series>, name: String) -> Vec<Series> {
    pair_series(series, name, |(Point(time, x), Point(_, y))| {
        Point(*time, x - y)
    })
}

pub fn alias(series: Vec<Series>, name: String) -> Vec<Series> {
    series
        .into_iter()
        .map(|Series { name: _, points }| Series {
            name: name.clone(),
            points,
        })
        .collect::<Vec<_>>()
}

pub fn alias_by_metric(series: Vec<Series>) -> Vec<Series> {
    series
        .into_iter()
        .map(|Series { name, points }| Series {
            name: base_metric(name),
            points,
        })
        .collect::<Vec<_>>()
}

pub fn alias_by_node(series: Vec<Series>, nodes: Vec<i64>) -> Vec<Series> {
    series
        .into_iter()
        .map(|Series { name, points }| Series {
            name: node_metric(name, nodes.clone()),
            points,
        })
        .collect::<Vec<_>>()
}

fn base_metric(metric: String) -> String {
    metric.split('.').last().unwrap_or(&metric).to_owned()
}

fn node_metric(metric: String, nodes: Vec<i64>) -> String {
    let count = metric.split('.').count();
    nodes
        .into_iter()
        .map(|node| {
            let number = if node < 0 {
                (count as i64 + node) as usize
            } else {
                node as usize
            };
            metric.split('.').nth(number).unwrap_or("").to_owned()
        })
        .fold(String::new(), |state, s| format!("{}.{}", state, s))
}
