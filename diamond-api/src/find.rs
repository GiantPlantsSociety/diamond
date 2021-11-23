use actix_web::error::ErrorInternalServerError;
use actix_web::web::{Data, Form, Json, Query};
use actix_web::{dev, FromRequest, HttpMessage, HttpRequest, HttpResponse, Result};
use futures::future::{FutureExt, LocalBoxFuture};
use serde::*;
use std::convert::From;
use std::str::FromStr;

use crate::context::Context;
use crate::parse::de_time_parse;
use crate::render_target::PathExpression;
use crate::storage::MetricResponseLeaf;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct MetricResponse {
    metrics: Vec<MetricResponseLeaf>,
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
                text: m.name.0.join("."),
                id: m.name.0.last().unwrap().to_owned(),
                allow_children: 0,
                expandable: 0,
                leaf: 1,
            }
        } else {
            JsonTreeLeaf {
                text: m.name.0.join("."),
                id: m.name.0.last().unwrap().to_owned(),
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
    #[serde(deserialize_with = "de_time_parse", default)]
    from: u32,
    #[serde(deserialize_with = "de_time_parse", default = "u32::max_value")]
    until: u32,
}

impl FromRequest for FindQuery {
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut dev::Payload) -> Self::Future {
        match req.content_type().to_lowercase().as_str() {
            "application/x-www-form-urlencoded" => Form::<FindQuery>::from_request(req, payload)
                .map(|r| Ok(r?.into_inner()))
                .boxed_local(),
            "application/json" => Json::<FindQuery>::from_request(req, payload)
                .map(|r| Ok(r?.into_inner()))
                .boxed_local(),
            _ => Query::<FindQuery>::from_request(req, payload)
                .map(|r| Ok(r?.into_inner()))
                .boxed_local(),
        }
    }
}

pub async fn find_handler(ctx: Data<Context>, query: FindQuery) -> Result<HttpResponse> {
    let path_expression =
        PathExpression::from_str(&query.query).map_err(ErrorInternalServerError)?;

    Ok(ctx.storage.find(&path_expression).map(|metrics| {
        if query.format == FindFormat::TreeJson {
            let metrics_json: Vec<JsonTreeLeaf> =
                metrics.into_iter().map(JsonTreeLeaf::from).collect();
            HttpResponse::Ok().json(metrics_json)
        } else {
            let metrics_completer = MetricResponse { metrics };
            HttpResponse::Ok().json(metrics_completer)
        }
    })?)
}

#[cfg(test)]
mod tests {
    use actix_web::test::TestRequest;
    use std::error::Error;

    use super::*;

    #[test]
    fn url_serialize() -> Result<(), Box<dyn Error>> {
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
    fn url_deserialize() -> Result<(), Box<dyn Error>> {
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
    fn url_deserialize_default() -> Result<(), Box<dyn Error>> {
        let params = FindQuery {
            query: "123".to_owned(),
            format: FindFormat::default(),
            wildcards: u8::default(),
            from: u32::default(),
            until: u32::max_value(),
        };

        assert_eq!(serde_urlencoded::from_str("query=123"), Ok(params));

        Ok(())
    }

    #[test]
    fn metric_response_convertion() {
        let mleaf: JsonTreeLeaf = MetricResponseLeaf {
            name: "123.456".parse().unwrap(),
            is_leaf: true,
        }
        .into();

        let leaf = JsonTreeLeaf {
            text: "123.456".to_owned(),
            id: "456".to_owned(),
            allow_children: 0,
            expandable: 0,
            leaf: 1,
        };

        assert_eq!(mleaf, leaf);

        let mleaf2: JsonTreeLeaf = MetricResponseLeaf {
            name: "123.456.789".parse().unwrap(),
            is_leaf: false,
        }
        .into();

        let leaf2 = JsonTreeLeaf {
            text: "123.456.789".to_owned(),
            id: "789".to_owned(),
            allow_children: 1,
            expandable: 1,
            leaf: 0,
        };

        assert_eq!(mleaf2, leaf2);
    }

    #[actix_rt::test]
    async fn find_request_parse_url() -> Result<(), actix_web::Error> {
        let r =
            TestRequest::with_uri("/find?query=123&format=treejson&wildcards=1&from=0&until=10")
                .to_srv_request();

        let (req, mut pl) = r.into_parts();

        let params = FindQuery {
            query: "123".to_owned(),
            format: FindFormat::TreeJson,
            wildcards: 1,
            from: 0,
            until: 10,
        };

        assert_eq!(FindQuery::from_request(&req, &mut pl).await?, params);
        Ok(())
    }

    #[actix_rt::test]
    async fn find_request_parse_form() -> Result<(), actix_web::Error> {
        let r = TestRequest::with_uri("/find")
            .append_header(("content-type", "application/x-www-form-urlencoded"))
            .set_payload("query=123&format=treejson&wildcards=1&from=0&until=10")
            .to_srv_request();

        let (req, mut pl) = r.into_parts();

        let params = FindQuery {
            query: "123".to_owned(),
            format: FindFormat::TreeJson,
            wildcards: 1,
            from: 0,
            until: 10,
        };

        assert_eq!(FindQuery::from_request(&req, &mut pl).await?, params);
        Ok(())
    }

    #[actix_rt::test]
    async fn find_request_parse_json() -> Result<(), actix_web::Error> {
        let r = TestRequest::with_uri("/render")
            .append_header(("content-type", "application/json"))
            .set_payload(
                r#"{"query":"123","format":"treejson","wildcards":1,"from":"0","until":"10"}"#,
            )
            .to_srv_request();

        let (req, mut pl) = r.into_parts();

        let params = FindQuery {
            query: "123".to_owned(),
            format: FindFormat::TreeJson,
            wildcards: 1,
            from: 0,
            until: 10,
        };

        assert_eq!(FindQuery::from_request(&req, &mut pl).await?, params);
        Ok(())
    }
}
