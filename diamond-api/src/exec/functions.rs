use super::{Point, Series};

fn aggregation_series(
    series: Vec<Series>,
    name: String,
    initial: f64,
    f: impl Fn((Point, Point)) -> Point,
) -> Vec<Series> {
    if series.len() == 0 {
        return series;
    }

    let init = series
        .iter()
        .next()
        .unwrap()
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

pub fn count_series(series: Vec<Series>, name: String) -> Vec<Series> {
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
