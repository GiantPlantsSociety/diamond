use super::{Point, Series};

fn aggregation_series(
    series: Vec<Series>,
    name: String,
    initial: f64,
    f: impl Fn((Point, Point)) -> Point,
) -> Option<Series> {
    series.iter().next().cloned().map(|first| {
        let init = first
            .points
            .iter()
            .map(|Point(time, _)| Point(*time, initial))
            .collect();

        Series {
            name,
            points: series.into_iter().fold(init, |acc: Vec<Point>, serie| {
                acc.into_iter()
                    .zip(serie.points.into_iter())
                    .map(&f)
                    .collect::<Vec<_>>()
            }),
        }
    })
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
    left: &Series,
    right: &Series,
    name: String,
    f: impl Fn((&Point, &Point)) -> Point,
) -> Series {
    Series {
        name,
        points: left
            .points
            .iter()
            .zip(right.points.iter())
            .map(&f)
            .collect::<Vec<_>>(),
    }
}

pub fn sum_series(series: Vec<Series>, name: String) -> Option<Series> {
    fn sum(pair: (Point, Point)) -> Point {
        let (x, y) = pair;
        Point(x.0, x.1 + y.1)
    };

    aggregation_series(series, name, 0.0, sum)
}

pub fn max_series(series: Vec<Series>, name: String) -> Option<Series> {
    fn max(pair: (Point, Point)) -> Point {
        let (x, y) = pair;
        Point(x.0, x.1.max(y.1))
    };

    aggregation_series(series, name, 0.0, max)
}

pub fn min_series(series: Vec<Series>, name: String) -> Option<Series> {
    fn min(pair: (Point, Point)) -> Point {
        let (x, y) = pair;
        Point(x.0, x.1.min(y.1))
    };

    aggregation_series(series, name, 0.0, min)
}

pub fn multiply_series(series: Vec<Series>, name: String) -> Option<Series> {
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
    adjust_series(sum_series(series, name).into_iter().collect(), avg)
}

pub fn divide_series(left: &Series, right: &Series, name: String) -> Series {
    pair_series(left, right, name, |(Point(time, x), Point(_, y))| {
        Point(*time, x / y)
    })
}

pub fn diff_series(left: &Series, right: &Series, name: String) -> Series {
    pair_series(left, right, name, |(Point(time, x), Point(_, y))| {
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

pub fn as_percent(series: Vec<Series>, total: Option<Series>, name: String) -> Vec<Series> {
    fn percent(pair: (&Point, &Point)) -> Point {
        let (x, y) = pair;
        Point(x.0, x.1 / y.1 * 100_f64)
    };

    let total_series = if let Some(total) = total {
        total
    } else {
        sum_series(series.clone(), "".to_owned()).unwrap()
    };

    series
        .iter()
        .map(|left| pair_series(left, &total_series, name.clone(), percent))
        .collect()
}
