use axum::{
    Router,
    extract::{Json, State},
    http::StatusCode,
    routing::post,
};
use dotenvy::dotenv;
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use store::KeyValueStore;

use std::env;
mod constants;
mod store;
mod wal_line;

#[derive(Deserialize)]
pub struct SetRequest {
    key: String,
    value: Value,
}

#[derive(Deserialize)]
pub struct GetDeleteRequest {
    key: String,
}

pub struct Server {
    pub is_leader: bool,
    pub port: u32,
    pub followers_ports: Vec<u32>,
    pub followers_base_url: String,
    pub wal_path: String,
}

impl Server {
    pub fn new(
        port: u32,
        followers_base_url: String,
        followers_ports: String,
        is_leader: bool,
        wal_path: String,
    ) -> Self {
        Self {
            port: port,
            followers_base_url: followers_base_url,
            followers_ports: Self::parse_ports(followers_ports),
            is_leader: is_leader,
            wal_path: wal_path,
        }
    }

    fn parse_ports(ports: String) -> Vec<u32> {
        let mut ret: Vec<u32> = Vec::new();
        for part in ports.split(",") {
            ret.push(part.trim().parse().unwrap());
        }
        ret
    }
}

pub struct AppState {
    pub store: Mutex<KeyValueStore>,
    pub server: Server,
}

type SharedState = Arc<AppState>;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let port: u32 = env::var("SERVER_PORT")
        .expect("SERVER_PORT must be set")
        .parse()
        .unwrap();
    let followers_base_url: String =
        env::var("FOLLOWERS_BASE_URL").expect("FOLLOWERS_BASE_URL must be set");
    let followers_ports: String = env::var("FOLLOWERS_PORTS").expect("FOLLOWERS_PORTS must be set");
    let is_leader: bool = match env::var("IS_LEADER") {
        Ok(val) => val.parse().unwrap(),
        Err(_e) => false,
    };
    let wal_path: String = env::var("WAL_PATH").expect("WAL_PATH must be set");

    let shared_state: SharedState = Arc::new(AppState {
        store: Mutex::new(KeyValueStore::new(wal_path.clone())),
        server: Server::new(
            port,
            followers_base_url,
            followers_ports,
            is_leader,
            wal_path,
        ),
    });
    {
        let mut store = shared_state.store.lock().unwrap();
        store.replay_wal();
    }
    let router: Router = Router::new()
        .route("/set", post(set_handler))
        .route("/get", post(get_handler))
        .route("/delete", post(delete_handler))
        .with_state(shared_state.clone());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", shared_state.server.port))
        .await
        .unwrap();

    axum::serve(listener, router).await.unwrap();
}

async fn set_handler(
    State(state): State<SharedState>,
    Json(payload): Json<SetRequest>,
) -> (StatusCode, String) {
    if !state.server.is_leader {
        return (
            StatusCode::BAD_REQUEST,
            "Followers accept reads only".to_string(),
        );
    }
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
    if !state.server.is_leader {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!("Followers accept reads only".to_string())),
        );
    }
    let mut store = state.store.lock().unwrap();
    if let Some(value) = store.delete(&payload.key) {
        (StatusCode::OK, Json(value))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({"error": "Not found"})))
    }
}
