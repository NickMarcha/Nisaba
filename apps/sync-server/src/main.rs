// Nisaba sync server — WebSocket + Yjs CRDT sync with SQLite persistence.

mod indexer;
mod persistence;

use tower_http::cors::CorsLayer;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path,
        State,
    },
    response::IntoResponse,
    routing::{get, put},
    Json,
    Router,
};
use futures_util::stream::{Stream, StreamExt};
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::sync::{broadcast, Mutex, RwLock};
use tracing::{error, info};
use yrs::{GetString, Text};
use yrs::updates::decoder::Decode;
use yrs::updates::encoder::{Encoder, EncoderV1};
use yrs::{ReadTxn, StateVector, Transact, Update};
use yrs::sync::protocol::{DefaultProtocol, Protocol};
use yrs::sync::Awareness;
use yrs_axum::broadcast::BroadcastGroup;
use yrs_axum::ws::AxumSink;
use futures_util::SinkExt;

use persistence::{spawn_persist, Persistence};
use indexer::{index_docs, index_to_json};

/// Wraps the WebSocket stream and yields only Binary/Text frames as Vec<u8>.
/// Skips Ping, Pong, and Close to avoid yrs decode errors on control frames.
struct YrsDataStream<S>(S);

impl<S> YrsDataStream<S> {
    fn new(inner: S) -> Self {
        Self(inner)
    }
}

impl<S, E> Stream for YrsDataStream<S>
where
    S: Stream<Item = Result<Message, E>> + Unpin,
    E: std::error::Error + Send + Sync + 'static,
{
    type Item = Result<Vec<u8>, yrs::sync::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match Pin::new(&mut self.0).poll_next(cx) {
                Poll::Ready(Some(Ok(msg))) => {
                    let data: Vec<u8> = match msg {
                        Message::Binary(d) => d.to_vec(),
                        Message::Text(t) => t.as_bytes().to_vec(),
                        Message::Ping(_) | Message::Pong(_) | Message::Close(_) => continue,
                    };
                    if data.is_empty() {
                        continue;
                    }
                    return Poll::Ready(Some(Ok(data)));
                }
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Some(Err(yrs::sync::Error::Other(Box::new(e)))));
                }
                Poll::Ready(None) => return Poll::Ready(None),
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

struct DocRoom {
    _subscription: Option<yrs::Subscription>,
    broadcast: Arc<BroadcastGroup>,
}

pub const META_DOC_ID: &str = "_meta";
pub const BLOCKS_DOC_ID: &str = "_blocks";

#[derive(Clone)]
struct AppState {
    doc_store: Arc<RwLock<HashMap<String, Arc<DocRoom>>>>,
    persistence: Arc<Persistence>,
    dirty_tx: broadcast::Sender<()>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "nisaba_sync_server=info,tower_http=info".into()),
        )
        .init();

    let db_path = std::env::var("NISABA_SYNC_DB").unwrap_or_else(|_| "sync.db".to_string());
    let persistence = Arc::new(
        Persistence::new(&db_path).expect("Failed to open SQLite database"),
    );
    info!("Persistence: {}", db_path);

    let (dirty_tx, _) = broadcast::channel(16);
    let app_state = AppState {
        doc_store: Arc::new(RwLock::new(HashMap::new())),
        persistence: persistence.clone(),
        dirty_tx: dirty_tx.clone(),
    };

    let state_for_rebuild = app_state.clone();
    tokio::spawn(async move {
        let mut rx = dirty_tx.subscribe();
        let mut dirty = false;
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(3));
        loop {
            tokio::select! {
                _ = rx.recv() => {
                    dirty = true;
                }
                _ = interval.tick() => {
                    if dirty {
                        dirty = false;
                        if let Err(e) = state_for_rebuild.rebuild_meta_and_blocks().await {
                            error!("rebuild_meta_and_blocks: {}", e);
                        }
                    }
                }
            }
        }
    });

    let app = Router::new()
        .route("/sync/{doc_id}", get(ws_handler))
        .route("/api/files", get(api_list_files))
        .route("/api/files/{doc_id}", get(api_get_file).put(api_put_file))
        .layer(CorsLayer::permissive())
        .with_state(app_state.clone());

    let addr = std::env::var("NISABA_SYNC_ADDR").unwrap_or_else(|_| "0.0.0.0:8765".to_string());
    let listener = tokio::net::TcpListener::bind(&addr).await.expect("Bind failed");
    info!("Sync server listening on {}", addr);

    if let Err(e) = app_state.rebuild_meta_and_blocks().await {
        error!("Initial rebuild_meta_and_blocks: {}", e);
    }

    axum::serve(listener, app).await.expect("Server failed");
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    Path(doc_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let doc_id = urlencoding::decode(&doc_id).unwrap_or(std::borrow::Cow::Borrowed(&doc_id));
    let doc_id = doc_id.to_string();
    if doc_id.is_empty() {
        return ws.on_upgrade(|_| async {}).into_response();
    }
    ws.on_upgrade(move |socket| peer(socket, doc_id, state))
        .into_response()
}

async fn api_list_files(State(state): State<AppState>) -> impl IntoResponse {
    match state.persistence.list_doc_ids() {
        Ok(ids) => Json(ids).into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn api_get_file(
    Path(doc_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let doc_id = urlencoding::decode(&doc_id).unwrap_or(std::borrow::Cow::Borrowed(&doc_id));
    let doc_id = doc_id.to_string();
    match state.materialize(&doc_id).await {
        Ok(Some(content)) => Json(serde_json::json!({ "content": content })).into_response(),
        Ok(None) => (axum::http::StatusCode::NOT_FOUND, "Document not found").into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn api_put_file(
    Path(doc_id): Path<String>,
    State(state): State<AppState>,
    Json(body): Json<serde_json::Value>,
) -> impl IntoResponse {
    let doc_id = urlencoding::decode(&doc_id).unwrap_or(std::borrow::Cow::Borrowed(&doc_id));
    let doc_id = doc_id.to_string();
    let content = body
        .get("content")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    match state.create_or_update_doc(&doc_id, &content).await {
        Ok(()) => axum::http::StatusCode::NO_CONTENT.into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn peer(ws: WebSocket, doc_id: String, state: AppState) {
    let room = state.get_or_create_room(&doc_id).await;
    if let Err(e) = room {
        error!("Failed to get room for doc {}: {}", doc_id, e);
        return;
    }
    let room = room.unwrap();

    let (sink, stream) = ws.split();
    let mut axum_sink = AxumSink::from(sink);

    // Send SyncStep1 + Awareness so the client replies with its state (SyncStep2).
    // Per y-sync client-server model: server must send SyncStep1 to receive client state.
    let start_payload: Result<Vec<u8>, yrs::sync::Error> = {
        let awareness = room.broadcast.awareness().read().await;
        let mut encoder = EncoderV1::new();
        DefaultProtocol
            .start(&*awareness, &mut encoder)
            .map(|_| encoder.to_vec())
    };
    if let Ok(payload) = start_payload {
        if !payload.is_empty() {
            if let Err(e) = axum_sink.send(payload).await {
                error!("Failed to send sync start for doc {}: {}", doc_id, e);
                return;
            }
        }
    }

    let sink = Arc::new(Mutex::new(axum_sink));
    let stream = YrsDataStream::new(stream);
    let sub = room.broadcast.subscribe(sink, stream);
    match sub.completed().await {
        Ok(_) => info!("Client disconnected from doc {}", doc_id),
        Err(e) => error!("Client connection error for doc {}: {}", doc_id, e),
    }
}

impl AppState {
    fn is_system_doc(doc_id: &str) -> bool {
        doc_id == META_DOC_ID || doc_id == BLOCKS_DOC_ID
    }

    async fn get_or_create_room(
        &self,
        doc_id: &str,
    ) -> Result<Arc<DocRoom>, Box<dyn std::error::Error + Send + Sync>> {
        {
            let store = self.doc_store.read().await;
            if let Some(room) = store.get(doc_id) {
                return Ok(Arc::clone(room));
            }
        }

        let mut store = self.doc_store.write().await;
        if let Some(room) = store.get(doc_id) {
            return Ok(Arc::clone(room));
        }

        let doc = yrs::Doc::new();
        let is_system = Self::is_system_doc(doc_id);
        if !is_system {
            if let Some(blob) = self.persistence.load(doc_id)? {
                if let Ok(update) = Update::decode_v1(&blob) {
                    doc.transact_mut().apply_update(update);
                }
            }
        }

        let awareness = Arc::new(RwLock::new(Awareness::new(doc)));
        let persistence = Arc::clone(&self.persistence);
        let doc_id_owned = doc_id.to_string();
        let awareness_for_persist = Arc::clone(&awareness);
        let dirty_tx = self.dirty_tx.clone();

        let subscription = if is_system {
            None
        } else {
            let mut aw = awareness.write().await;
            let sub = aw
                .doc_mut()
                .observe_update_v1(move |_txn, _u| {
                    let persistence = Arc::clone(&persistence);
                    let doc_id = doc_id_owned.clone();
                    let awareness = Arc::clone(&awareness_for_persist);
                    let dirty_tx = dirty_tx.clone();
                    tokio::spawn(async move {
                        let aw_guard = awareness.read().await;
                        let full_state = aw_guard
                            .doc()
                            .transact()
                            .encode_diff_v1(&StateVector::default());
                        drop(aw_guard);
                        spawn_persist(persistence, doc_id, full_state);
                        let _ = dirty_tx.send(());
                    });
                })
                .map_err(|e| format!("observe: {:?}", e))?;
            Some(sub)
        };

        let broadcast = Arc::new(BroadcastGroup::new(awareness, 32).await);
        let room = Arc::new(DocRoom {
            _subscription: subscription,
            broadcast,
        });
        store.insert(doc_id.to_string(), Arc::clone(&room));
        Ok(room)
    }

    async fn rebuild_meta_and_blocks(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let doc_ids = self.persistence.list_doc_ids()?;
        let doc_ids: Vec<String> = doc_ids
            .into_iter()
            .filter(|id| !Self::is_system_doc(id))
            .collect();

        let meta_json = serde_json::to_string(&doc_ids)?;
        self.update_system_doc(META_DOC_ID, &meta_json).await?;

        let mut docs = Vec::new();
        for id in &doc_ids {
            if let Ok(Some(content)) = self.materialize(id).await {
                docs.push((id.clone(), content));
            }
        }
        let (blocks, links) = index_docs(&docs);
        let blocks_json = index_to_json(&blocks, &links);
        self.update_system_doc(BLOCKS_DOC_ID, &blocks_json).await?;

        info!("Rebuilt index: {} files, {} blocks, {} links", doc_ids.len(), blocks.len(), links.len());
        Ok(())
    }

    async fn update_system_doc(
        &self,
        doc_id: &str,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let room = self.get_or_create_room(doc_id).await?;
        let len = {
            let aw = room.broadcast.awareness().read().await;
            let doc = aw.doc();
            let text = doc.get_or_insert_text("content");
            let txn = doc.transact();
            text.len(&txn)
        };

        let mut aw = room.broadcast.awareness().write().await;
        let doc = aw.doc_mut();
        let text = doc.get_or_insert_text("content");
        let mut txn = doc.transact_mut();
        text.remove_range(&mut txn, 0, len);
        text.insert(&mut txn, 0, content);
        Ok(())
    }

    /// Materialize Markdown from Yjs state (from persistence or in-memory doc).
    async fn materialize(&self, doc_id: &str) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        let blob = if let Some(room) = self.doc_store.read().await.get(doc_id) {
            let aw = room.broadcast.awareness().read().await;
            let encoded = aw.doc().transact().encode_diff_v1(&StateVector::default());
            Some(encoded)
        } else {
            self.persistence.load(doc_id)?
        };
        let Some(blob) = blob else {
            return Ok(None);
        };
        let update = Update::decode_v1(&blob).map_err(|e| format!("decode: {:?}", e))?;
        let doc = yrs::Doc::new();
        doc.transact_mut().apply_update(update);
        let text = doc.get_or_insert_text("content");
        let s = text.get_string(&doc.transact());
        Ok(Some(s))
    }

    /// Create or update a doc with Markdown content and persist.
    async fn create_or_update_doc(
        &self,
        doc_id: &str,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let doc = yrs::Doc::new();
        let text = doc.get_or_insert_text("content");
        if !content.is_empty() {
            text.insert(&mut doc.transact_mut(), 0, content);
        }
        let blob = doc.transact().encode_diff_v1(&StateVector::default());
        self.persistence.save(doc_id, &blob)?;
        let _ = self.dirty_tx.send(());
        Ok(())
    }
}
