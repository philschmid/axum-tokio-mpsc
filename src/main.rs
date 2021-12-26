use axum::{
    extract::Extension, http::Method, response::IntoResponse, routing::post, AddExtensionLayer,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use tokio::sync::{
    mpsc::{self, Sender},
    oneshot,
};
use tower::ServiceBuilder;
use tower_http::{
    cors::{any, CorsLayer},
    trace::TraceLayer,
};
use tracing::info;

mod compute_heavy;
// the input to our `create_user` handler
#[derive(Deserialize, Debug)]
pub struct PredictRequest {
    // inputs: String,
    inputs: i32,
}

pub struct MpscPayload {
    payload: PredictRequest,
    resp: oneshot::Sender<i32>,
}

#[derive(Serialize, Debug)]
struct PredictResponse {
    value: i32,
}

async fn compute_route(
    Json(payload): Json<PredictRequest>,
    Extension(tx_clone): Extension<Sender<MpscPayload>>,
) -> impl IntoResponse {
    // create oneshot channel for recieving response from compute_heavy
    let (resp_tx, resp_rx) = oneshot::channel::<i32>();

    // send data here
    tracing::debug!("Got {}", payload.inputs);
    let _ = tx_clone
        .send(MpscPayload {
            payload,
            resp: resp_tx,
        })
        .await;
    // Await the response
    let res = resp_rx.await;

    Json(PredictResponse {
        value: res.unwrap(),
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (tx, rx) = mpsc::channel::<MpscPayload>(32);

    // Set the RUST_LOG, if it hasn't been explicitly defined
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "axum_tokio_mpsc=debug,tower_http=debug")
    }
    // init tracing for `TraceLayer`
    tracing_subscriber::fmt::init();

    // MPSC channel consumer for prediction
    tokio::spawn(async move { compute_heavy::heavy_computation(rx).await });

    // Server middlewar stack
    let middleware_stack = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(
            // see https://docs.rs/tower-http/latest/tower_http/cors/index.html
            // for more details
            CorsLayer::new()
                .allow_origin(any())
                .allow_methods(vec![Method::POST, Method::OPTIONS])
                .allow_headers(any()),
        )
        .layer(AddExtensionLayer::new(tx.clone()));

    // build our application with a route
    let app = Router::new()
        .route("/predict", post(compute_route))
        .layer(middleware_stack);

    // run it with hyper on localhost:3000
    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .http1_preserve_header_case(true)
        .http1_title_case_headers(true)
        .serve(app.into_make_service())
        .await
        .unwrap();
    Ok(())
}
