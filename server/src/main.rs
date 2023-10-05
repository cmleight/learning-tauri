mod auth;

use std::collections::HashMap;
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use sqlx::{Executor, Pool, Postgres};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use sqlx::postgres::PgPoolOptions;
use warp::ws::{Message, WebSocket};
use warp::Filter;

#[tokio::main]
async fn main() {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgres://postgres:postgres@localhost").await?;
    pretty_env_logger::init();

    let db = warp::any().map(move || pool.clone());

    let login = warp::path("/login")
        .and(warp::post())
        .and(warp::body::json())
        .and(db)
        .and_then(login);
    let create = warp::path("/create")
        .and(warp::post())
        .and(warp::body::json())
        .and(db)
        .and_then(create);

    let chat = warp::path("chat")
        .and(warp::ws())
        .and(db)
        .map(|ws: warp::ws::Ws, db| {
            // This will call our function if the handshake succeeds.
            ws.on_upgrade(move |socket| user_connected(socket, db))
        });

    // GET / -> index html
    let index = warp::path::end().map(|| warp::reply::html(INDEX_HTML));

    let routes = index.or(chat).or(login).or(create);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn login(login: HashMap<String, String>, db: Pool<Postgres>) -> Result<impl warp::Reply, warp::Rejection> {
    let mut conn = db.acquire().await?;
    if login.get("username").is_none() || login.get("password").is_none() {
        return Ok(warp::reply::json(&auth::User { id: 0, username: "".to_string(), password: "".to_string() }));
    }
    let user = sqlx::query_as::<_, auth::User>("SELECT * FROM users WHERE username = $1 AND password = $2")
        .bind(login.get("username").unwrap())
        .bind(login.get("password").unwrap())
        .fetch_one(&mut conn)
        .await?;
    Ok(warp::reply::json(&user))
}

async fn create_user(user: HashMap<String, String>, db: Pool<Postgres>) -> Result<impl warp::Reply, warp::Rejection> {
    let mut conn = db.acquire().await?;
    if user.get("username").is_none() || user.get("password").is_none() {
        return Ok(warp::reply::json(&auth::User { id: 0, username: "".to_string(), password: "".to_string() }));
    }
    let user = sqlx::query_as::<_, auth::User>("INSERT INTO users (username, password) VALUES ($1, $2) RETURNING *")
        .bind(user.get("username").unwrap())
        .bind(user.get("password").unwrap())
        .fetch_one(&mut conn)
        .await?;
    Ok(warp::reply::json(&user))
}

async fn user_connected(ws: WebSocket, db: Pool<Postgres>) {
    // Split the socket into a sender and receive of messages.
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (tx, rx) = mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);

    tokio::task::spawn(async move {
        while let Some(message) = rx.next().await {
            user_ws_tx
                .send(message)
                .unwrap_or_else(|e| {
                    eprintln!("websocket send error: {}", e);
                })
                .await;
        }
    });

    // Save the sender in our list of connected users.
    users.write().await.insert(my_id, tx);

    // Return a `Future` that is basically a state machine managing
    // this specific user's connection.

    // Every time the user sends a message, broadcast it to
    // all other users...
    while let Some(result) = user_ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("websocket error(uid={}): {}", my_id, e);
                break;
            }
        };
        user_message(my_id, msg, &users).await;
    }

    // user_ws_rx stream will keep processing as long as the user stays
    // connected. Once they disconnect, then...
    user_disconnected(my_id, &users).await;
}

async fn user_message(my_id: usize, msg: Message, users: &Users) {
    // Skip any non-Text messages...
    let msg = if let Ok(s) = msg.to_str() {
        s
    } else {
        return;
    };

    let new_msg = format!("<User#{}>: {}", my_id, msg);

    // New message from this user, send it to everyone else (except same uid)...
    for (&uid, tx) in users.read().await.iter() {
        if my_id != uid {
            if let Err(_disconnected) = tx.send(Message::text(new_msg.clone())) {
                // The tx is disconnected, our `user_disconnected` code
                // should be happening in another task, nothing more to
                // do here.
            }
        }
    }
}

async fn user_disconnected(my_id: usize, users: &Users) {
    eprintln!("good bye user: {}", my_id);

    // Stream closed up, so remove from the user list
    users.write().await.remove(&my_id);
}

static INDEX_HTML: &str = include_str!("../static/index.html");
