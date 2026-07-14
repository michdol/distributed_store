use axum::{
    Router,
    extract::{Json, State},
    http::StatusCode,
    routing::post,
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use store::KeyValueStore;

mod constants;
mod store;

#[derive(Deserialize)]
pub struct SetRequest {
    key: String,
    value: Value,
}

#[derive(Deserialize)]
pub struct GetDeleteRequest {
    key: String,
}

pub struct AppState {
    pub store: Mutex<KeyValueStore>,
}

type SharedState = Arc<AppState>;

#[tokio::main]
async fn main() {
    let shared_state: SharedState = Arc::new(AppState {
        store: Mutex::new(KeyValueStore::new("./src/wal.txt".to_string())),
    });
    {
        let mut store = shared_state.store.lock().unwrap();
        store.replay_wal();
    }
    let router: Router = Router::new()
        .route("/set", post(set_handler))
        .route("/get", post(get_handler))
        .route("/delete", post(delete_handler))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, router).await.unwrap();
}

async fn set_handler(
    State(state): State<SharedState>,
    Json(payload): Json<SetRequest>,
) -> (StatusCode, String) {
    let mut store = state.store.lock().unwrap();
    store.set(payload.key, payload.value);
    (StatusCode::OK, "Ok".to_string())
}

async fn get_handler(
    State(state): State<SharedState>,
    Json(payload): Json<GetDeleteRequest>,
) -> (StatusCode, Json<Value>) {
    let store = state.store.lock().unwrap();
    if let Some(value) = store.get(&payload.key) {
        (StatusCode::OK, Json(value))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "Not found"})))
    }
}

async fn delete_handler(
    State(state): State<SharedState>,
    Json(payload): Json<GetDeleteRequest>,
) -> (StatusCode, Json<Value>) {
    let mut store = state.store.lock().unwrap();
    if let Some(value) = store.delete(&payload.key) {
        (StatusCode::OK, Json(value))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "Not found"})))
    }
}
