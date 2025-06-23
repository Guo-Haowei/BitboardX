use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};
use serde::{Deserialize, Serialize};
use std::{fs, process::Stdio};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::Command,
};

mod meta;

// -------- Types --------

#[derive(Serialize, Deserialize)]
struct PgnRequest {
    file: String,
}

#[derive(Serialize, Deserialize)]
struct FenRequest {
    fen: String,
}

// -------- Routes --------

#[get("/meta")]
async fn get_meta() -> impl Responder {
    let meta = meta::get_meta_impl();
    HttpResponse::Ok().json(meta)
}

#[post("/pgn")]
async fn post_pgn(req: web::Json<PgnRequest>) -> impl Responder {
    let path = format!("pgn/{}", req.file);
    match fs::read_to_string(path) {
        Ok(content) => HttpResponse::Ok().body(content),
        Err(_) => HttpResponse::NotFound().body("PGN file not found"),
    }
}

#[post("/bestmove")]
async fn post_bestmove(req: web::Json<FenRequest>) -> impl Responder {
    match get_best_move_from_uci(&req.fen).await {
        Ok(bestmove) => HttpResponse::Ok().json(serde_json::json!({ "bestmove": bestmove })),
        Err(e) => HttpResponse::InternalServerError().body(format!("UCI error: {e}")),
    }
}

// -------- UCI Engine Integration --------

async fn get_best_move_from_uci(fen: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut child =
        Command::new("stockfish").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()?;

    let mut stdin = child.stdin.take().unwrap();
    let stdout = BufReader::new(child.stdout.take().unwrap());
    let mut lines = stdout.lines();

    // UCI init
    stdin.write_all(b"uci\nisready\n").await?;

    while let Some(line) = lines.next_line().await? {
        if line.trim() == "readyok" {
            break;
        }
    }

    stdin.write_all(format!("position fen {}\ngo depth 12\n", fen).as_bytes()).await?;

    while let Some(line) = lines.next_line().await? {
        if line.starts_with("bestmove") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            return Ok(parts[1].to_string());
        }
    }

    Err("No bestmove received".into())
}

// -------- Main --------

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let root_path = env!("CARGO_MANIFEST_DIR");
    println!("Project root: {}", root_path);
    println!("Server running on http://localhost:3000");

    HttpServer::new(|| {
        let cors = Cors::default()
            // .allow_any_origin()
            .allowed_origin("http://localhost:8001") // the frontend is running on port 8001
            .allow_any_method()
            .allow_any_header();

        App::new().wrap(cors).service(get_meta).service(post_pgn).service(post_bestmove)
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
