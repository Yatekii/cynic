//! HTTP client support for cynic.
//!
//! These are hidden behind feature flags by default as HTTP clients are quite
//! heavy dependencies, and there's several options to choose from.

#[cfg(feature = "surf")]
#[cfg_attr(docsrs, doc(cfg(feature = "surf")))]
pub use self::surf_ext::SurfExt;

#[cfg(feature = "reqwest")]
#[cfg_attr(docsrs, doc(cfg(feature = "reqwest")))]
pub use reqwest_ext::ReqwestExt;

#[cfg(feature = "reqwest-blocking")]
#[cfg_attr(docsrs, doc(cfg(feature = "reqwest-blocking")))]
pub use reqwest_blocking_ext::ReqwestBlockingExt;

#[cfg(feature = "reqwasm")]
#[cfg_attr(docsrs, doc(cfg(feature = "reqwasm")))]
pub use reqwasm_ext::ReqwasmExt;
use wasm_bindgen::JsValue;

#[cfg(feature = "surf")]
mod surf_ext {
    use serde_json::json;
    use std::{future::Future, pin::Pin};

    use crate::{GraphQlResponse, Operation};

    type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

    /// An extension trait for surf::RequestBuilder.
    ///
    /// ```rust,no_run
    /// # mod schema {
    /// #   cynic::use_schema!("../schemas/starwars.schema.graphql");
    /// # }
    /// #
    /// # #[derive(cynic::QueryFragment)]
    /// # #[cynic(
    /// #    schema_path = "../schemas/starwars.schema.graphql",
    /// #    schema_module = "schema",
    /// # )]
    /// # struct Film {
    /// #    title: Option<String>,
    /// #    director: Option<String>
    /// # }
    /// #
    /// # #[derive(cynic::QueryFragment)]
    /// # #[cynic(
    /// #     schema_path = "../schemas/starwars.schema.graphql",
    /// #     schema_module = "schema",
    /// #     graphql_type = "Root"
    /// # )]
    /// # struct FilmDirectorQuery {
    /// #     #[arguments(id = cynic::Id::new("ZmlsbXM6MQ=="))]
    /// #     film: Option<Film>,
    /// # }
    /// use cynic::{http::SurfExt, QueryBuilder};
    ///
    /// # async move {
    /// let operation = FilmDirectorQuery::build(());
    ///
    /// let response = surf::post("https://swapi-graphql.netlify.app/.netlify/functions/index")
    ///     .run_graphql(operation)
    ///     .await
    ///     .unwrap();
    ///
    /// println!(
    ///     "The director is {}",
    ///     response.data
    ///         .and_then(|d| d.film)
    ///         .and_then(|f| f.director)
    ///         .unwrap()
    /// );
    /// # };
    /// ```
    #[cfg_attr(docsrs, doc(cfg(feature = "surf")))]
    pub trait SurfExt {
        /// Runs a GraphQL query with the parameters in RequestBuilder, decodes
        /// the and returns the result.
        ///
        /// If a `json_decode::Error` occurs it can be obtained via downcast_ref on
        /// the `surf::Error`.
        fn run_graphql<'a, ResponseData: 'a>(
            self,
            operation: Operation<'a, ResponseData>,
        ) -> BoxFuture<'a, Result<GraphQlResponse<ResponseData>, surf::Error>>;
    }

    impl SurfExt for surf::RequestBuilder {
        fn run_graphql<'a, ResponseData: 'a>(
            self,
            operation: Operation<'a, ResponseData>,
        ) -> BoxFuture<'a, Result<GraphQlResponse<ResponseData>, surf::Error>> {
            Box::pin(async move {
                self.body(json!(&operation))
                    .recv_json::<GraphQlResponse<serde_json::Value>>()
                    .await
                    .and_then(|response| operation.decode_response(response).map_err(|e| e.into()))
            })
        }
    }
}

#[cfg(any(feature = "reqwest", feature = "reqwest-blocking"))]
#[derive(thiserror::Error, Debug)]
pub enum CynicReqwestError {
    #[error("Error making HTTP request: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Error decoding GraphQL response: {0}")]
    DecodeError(#[from] json_decode::DecodeError),
}

#[cfg(feature = "reqwest")]
mod reqwest_ext {
    use super::CynicReqwestError;
    use std::{future::Future, pin::Pin};

    use crate::{GraphQlResponse, Operation};

    type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

    /// An extension trait for reqwest::RequestBuilder.
    ///
    /// ```rust,no_run
    /// # mod schema {
    /// #   cynic::use_schema!("../schemas/starwars.schema.graphql");
    /// # }
    /// #
    /// # #[derive(cynic::QueryFragment)]
    /// # #[cynic(
    /// #    schema_path = "../schemas/starwars.schema.graphql",
    /// #    schema_module = "schema",
    /// # )]
    /// # struct Film {
    /// #    title: Option<String>,
    /// #    director: Option<String>
    /// # }
    /// #
    /// # #[derive(cynic::QueryFragment)]
    /// # #[cynic(
    /// #     schema_path = "../schemas/starwars.schema.graphql",
    /// #     schema_module = "schema",
    /// #     graphql_type = "Root"
    /// # )]
    /// # struct FilmDirectorQuery {
    /// #     #[arguments(id = cynic::Id::new("ZmlsbXM6MQ=="))]
    /// #     film: Option<Film>,
    /// # }
    /// use cynic::{http::ReqwestExt, QueryBuilder};
    ///
    /// # async move {
    /// let operation = FilmDirectorQuery::build(());
    ///
    /// let client = reqwest::Client::new();
    /// let response = client.post("https://swapi-graphql.netlify.app/.netlify/functions/index")
    ///     .run_graphql(operation)
    ///     .await
    ///     .unwrap();
    ///
    /// println!(
    ///     "The director is {}",
    ///     response.data
    ///         .and_then(|d| d.film)
    ///         .and_then(|f| f.director)
    ///         .unwrap()
    /// );
    /// # };
    /// ```
    #[cfg_attr(docsrs, doc(cfg(feature = "reqwest")))]
    pub trait ReqwestExt {
        /// Runs a GraphQL query with the parameters in RequestBuilder, decodes
        /// the and returns the result.
        ///
        /// If a `json_decode::Error` occurs it can be obtained via downcast_ref on
        /// the `surf::Error`.
        fn run_graphql<'a, ResponseData: 'a>(
            self,
            operation: Operation<'a, ResponseData>,
        ) -> BoxFuture<'a, Result<GraphQlResponse<ResponseData>, CynicReqwestError>>;
    }

    impl ReqwestExt for reqwest::RequestBuilder {
        fn run_graphql<'a, ResponseData: 'a>(
            self,
            operation: Operation<'a, ResponseData>,
        ) -> BoxFuture<'a, Result<GraphQlResponse<ResponseData>, CynicReqwestError>> {
            Box::pin(async move {
                match self
                    .json(&operation)
                    .send()
                    //.recv_json::<GraphQlResponse<serde_json::Value>>()
                    .await
                {
                    Ok(response) => response
                        .json::<GraphQlResponse<serde_json::Value>>()
                        .await
                        .map_err(CynicReqwestError::ReqwestError)
                        .and_then(|gql_response| {
                            operation
                                .decode_response(gql_response)
                                .map_err(CynicReqwestError::DecodeError)
                        }),
                    Err(e) => Err(CynicReqwestError::ReqwestError(e)),
                }
            })
        }
    }
}

#[cfg(feature = "reqwest-blocking")]
mod reqwest_blocking_ext {
    use super::CynicReqwestError;

    use crate::{GraphQlResponse, Operation};

    /// An extension trait for reqwest::blocking::RequestBuilder.
    ///
    /// ```rust,no_run
    /// # mod schema {
    /// #   cynic::use_schema!("../schemas/starwars.schema.graphql");
    /// # }
    /// #
    /// # #[derive(cynic::QueryFragment)]
    /// # #[cynic(
    /// #    schema_path = "../schemas/starwars.schema.graphql",
    /// #    schema_module = "schema",
    /// # )]
    /// # struct Film {
    /// #    title: Option<String>,
    /// #    director: Option<String>
    /// # }
    /// #
    /// # #[derive(cynic::QueryFragment)]
    /// # #[cynic(
    /// #     schema_path = "../schemas/starwars.schema.graphql",
    /// #     schema_module = "schema",
    /// #     graphql_type = "Root"
    /// # )]
    /// # struct FilmDirectorQuery {
    /// #     #[arguments(id = cynic::Id::new("ZmlsbXM6MQ=="))]
    /// #     film: Option<Film>,
    /// # }
    /// use cynic::{http::ReqwestBlockingExt, QueryBuilder};
    ///
    /// let operation = FilmDirectorQuery::build(());
    ///
    /// let client = reqwest::blocking::Client::new();
    /// let response = client.post("https://swapi-graphql.netlify.app/.netlify/functions/index")
    ///     .run_graphql(operation)
    ///     .unwrap();
    ///
    /// println!(
    ///     "The director is {}",
    ///     response.data
    ///         .and_then(|d| d.film)
    ///         .and_then(|f| f.director)
    ///         .unwrap()
    /// );
    /// ```
    #[cfg_attr(docsrs, doc(cfg(feature = "reqwest-blocking")))]
    pub trait ReqwestBlockingExt {
        /// Runs a GraphQL query with the parameters in RequestBuilder, decodes
        /// the and returns the result.
        ///
        /// If a `json_decode::Error` occurs it can be obtained via downcast_ref on
        /// the `surf::Error`.
        fn run_graphql<'a, ResponseData: 'a>(
            self,
            operation: Operation<'a, ResponseData>,
        ) -> Result<GraphQlResponse<ResponseData>, CynicReqwestError>;
    }

    impl ReqwestBlockingExt for reqwest::blocking::RequestBuilder {
        fn run_graphql<'a, ResponseData: 'a>(
            self,
            operation: Operation<'a, ResponseData>,
        ) -> Result<GraphQlResponse<ResponseData>, CynicReqwestError> {
            self.json(&operation)
                .send()
                .and_then(|response| response.json::<GraphQlResponse<serde_json::Value>>())
                .map_err(CynicReqwestError::ReqwestError)
                .and_then(|gql_response| {
                    operation
                        .decode_response(gql_response)
                        .map_err(CynicReqwestError::DecodeError)
                })
        }
    }
}

#[cfg(feature = "reqwasm")]
#[derive(thiserror::Error, Debug)]
pub enum CynicReqwasmError {
    #[error("Error making HTTP request: {0}")]
    Reqwasm(#[from] reqwasm::Error),
    #[error("Error decoding GraphQL response: {0}")]
    Decode(#[from] json_decode::DecodeError),
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Error while interfacing JS")]
    Js(JsValue),
}

#[cfg(feature = "reqwasm")]
mod reqwasm_ext {
    use reqwasm::http::FormData;
    use std::{future::Future, pin::Pin};
    use wasm_bindgen::JsValue;

    use super::CynicReqwasmError;
    use crate::{GraphQlResponse, Operation};

    type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

    /// An extension trait for reqwasm::http::Request.
    ///
    /// ```rust,no_run
    /// # mod schema {
    /// #   cynic::use_schema!("../schemas/starwars.schema.graphql");
    /// # }
    /// #
    /// # #[derive(cynic::QueryFragment)]
    /// # #[cynic(
    /// #    schema_path = "../schemas/starwars.schema.graphql",
    /// #    schema_module = "schema",
    /// # )]
    /// # struct Film {
    /// #    title: Option<String>,
    /// #    director: Option<String>
    /// # }
    /// #
    /// # #[derive(cynic::QueryFragment)]
    /// # #[cynic(
    /// #     schema_path = "../schemas/starwars.schema.graphql",
    /// #     schema_module = "schema",
    /// #     graphql_type = "Root"
    /// # )]
    /// # struct FilmDirectorQuery {
    /// #     #[arguments(id = cynic::Id::new("ZmlsbXM6MQ=="))]
    /// #     film: Option<Film>,
    /// # }
    /// use cynic::{http::ReqwasmExt, QueryBuilder};
    ///
    /// # async move {
    /// let operation = FilmDirectorQuery::build(());
    ///
    /// let response = reqwasm::http::Request::post("https://swapi-graphql.netlify.app/.netlify/functions/index")
    ///     .run_graphql(operation)
    ///     .await
    ///     .unwrap();
    ///
    /// println!(
    ///     "The director is {}",
    ///     response.data
    ///         .and_then(|d| d.film)
    ///         .and_then(|f| f.director)
    ///         .unwrap()
    /// );
    /// # };
    /// ```
    #[cfg_attr(docsrs, doc(cfg(feature = "reqwasm")))]
    pub trait ReqwasmExt {
        /// Runs a GraphQL query with the parameters in RequestBuilder, decodes
        /// the and returns the result.
        ///
        /// If a `json_decode::Error` occurs it can be obtained via downcast_ref on
        /// the `reqwasm::Error`.
        fn run_graphql<'a, ResponseData: 'a>(
            self,
            operation: Operation<'a, ResponseData>,
        ) -> BoxFuture<'a, Result<GraphQlResponse<ResponseData>, CynicReqwasmError>>;
    }

    impl ReqwasmExt for reqwasm::http::Request {
        fn run_graphql<'a, ResponseData: 'a>(
            self,
            operation: Operation<'a, ResponseData>,
        ) -> BoxFuture<'a, Result<GraphQlResponse<ResponseData>, CynicReqwasmError>> {
            Box::pin(async move {
                let form_data = FormData::new().map_err(CynicReqwasmError::Js)?;

                let operations = JsValue::from_str(&serde_json::to_string(&operation)?);

                let map = JsValue::from_str(&serde_json::to_string(&operation.file_map())?);

                form_data
                    .append_with_blob("operations", &operations.into())
                    .map_err(CynicReqwasmError::Js)?;
                form_data
                    .append_with_blob("map", &map.clone().into())
                    .map_err(CynicReqwasmError::Js)?;
                for (i, file) in operation.files.iter().enumerate() {
                    let file = gloo_file::File::new(&file.1.name, file.1.content.as_slice());
                    form_data
                        .append_with_blob(&format!("{}", i), &file.as_ref())
                        .map_err(CynicReqwasmError::Js)?;
                }

                match self.body(form_data).send().await {
                    Ok(response) => {
                        let response = response.json::<GraphQlResponse<serde_json::Value>>().await;
                        response
                            .map_err(CynicReqwasmError::Reqwasm)
                            .and_then(|gql_response| {
                                operation
                                    .decode_response(gql_response)
                                    .map_err(CynicReqwasmError::Decode)
                            })
                    }
                    Err(error) => Err(CynicReqwasmError::Reqwasm(error)),
                }
            })
        }
    }
}
