use actix_web::{Form, HttpResponse, Json, Query, State};
use failure::*;
use serde::*;

use crate::opts::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderQuery {
    target: Vec<String>,
    format: String,
    from: u32,
    until: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderPoint(f64, u32);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderResponceEntry {
    target: String,
    datapoints: Vec<RenderPoint>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderResponce {
    entries: Vec<RenderResponceEntry>,
}

fn render_any(args: &Args, params: &RenderQuery) -> Result<HttpResponse, Error> {
    let dir = &args.path;
    Ok(HttpResponse::Ok().finish())
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
            format: "json".to_owned(),
            target: ["app.numUsers".to_owned()].to_vec(),
            from: 0,
            until: 10,
        };

        assert_eq!(
            "target=app.numUsers&format=json&from=0&until=10",
            serde_urlencoded::to_string(params.clone())?
        );

        Ok(())
    }

    #[test]
    fn url_serialize_multiple() -> Result<(), Error> {
        let params = RenderQuery {
            format: "json".to_owned(),
            target: ["app.numUsers".to_owned(), "app.numServers".to_owned()].to_vec(),
            from: 0,
            until: 10,
        };

        assert_eq!(
            "target=app.numUsers&target=app.numServers&format=json&from=0&until=10",
            serde_urlencoded::to_string(params.clone())?
        );

        Ok(())
    }

    #[test]
    fn response() {
        let rd = to_string(
            &[RenderResponceEntry {
                target: "entries".into(),
                datapoints: [
                    RenderPoint(1.0, 1311836008),
                    RenderPoint(2.0, 1311836009),
                    RenderPoint(3.0, 1311836010),
                    RenderPoint(5.0, 1311836011),
                    RenderPoint(6.0, 1311836012),
                ]
                .to_vec(),
            }]
            .to_vec(),
        )
        .unwrap();

        let rs =
            r#"[{"target":"entries","datapoints":[[1.0,1311836008],[2.0,1311836009],[3.0,1311836010],[5.0,1311836011],[6.0,1311836012]]}]"#;

        assert_eq!(rd, rs);
    }
}
