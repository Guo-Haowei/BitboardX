use futures::{SinkExt, StreamExt};
use std::time::Duration;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Listening on ws://127.0.0.1:3000");

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            let ws_stream = accept_async(stream).await.unwrap();
            let (mut ws_sink, mut ws_stream) = ws_stream.split();

            // Wait for "start" message from client
            while let Some(Ok(msg)) = ws_stream.next().await {
                if let Message::Text(text) = msg {
                    if text == "start" {
                        println!("Received start from client");

                        let moves = ["e2e4", "e7e5", "g1f3", "b8c6"];
                        for mv in moves {
                            if ws_sink.send(Message::Text(mv.into())).await.is_err() {
                                break;
                            }
                            tokio::time::sleep(Duration::from_secs(1)).await;
                        }
                        break;
                    }
                }
            }
        });
    }
}

// use actix_cors::Cors;
// use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};
// use async_std::stream::StreamExt;
// use serde::{Deserialize, Serialize};
// use std::path::Path;
// use std::{fs, process::Stdio};
// use tokio::{
//     io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
//     process::Command,
// };

// mod meta;

// // -------- Types --------

// #[derive(Serialize, Deserialize)]
// struct PgnRequest {
//     file: String,
// }

// #[derive(Serialize, Deserialize)]
// struct FenRequest {
//     fen: String,
// }

// // -------- Routes --------

// #[get("/meta")]
// async fn get_meta() -> impl Responder {
//     let match_dir = Path::new(get_root_path()).join("matches");
//     let match_dir = match_dir.as_path();
//     let data = meta::get_meta_impl(&match_dir).unwrap();
//     // println!("Meta data: {:?}", data);
//     HttpResponse::Ok().json(data)
// }

// #[post("/pgn")]
// async fn post_pgn(req: web::Json<PgnRequest>) -> impl Responder {
//     let path = format!("pgn/{}", req.file);
//     match fs::read_to_string(path) {
//         Ok(content) => HttpResponse::Ok().body(content),
//         Err(_) => HttpResponse::NotFound().body("PGN file not found"),
//     }
// }

// #[post("/bestmove")]
// async fn post_bestmove(req: web::Json<FenRequest>) -> impl Responder {
//     match get_best_move_from_uci(&req.fen).await {
//         Ok(bestmove) => HttpResponse::Ok().json(serde_json::json!({ "bestmove": bestmove })),
//         Err(e) => HttpResponse::InternalServerError().body(format!("UCI error: {e}")),
//     }
// }

// // -------- UCI Engine Integration --------

// async fn get_best_move_from_uci(fen: &str) -> Result<String, Box<dyn std::error::Error>> {
//     let mut child =
//         Command::new("stockfish").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn()?;

//     let mut stdin = child.stdin.take().unwrap();
//     let stdout = BufReader::new(child.stdout.take().unwrap());
//     let mut lines = stdout.lines();

//     // UCI init
//     stdin.write_all(b"uci\nisready\n").await?;

//     while let Some(line) = lines.next_line().await? {
//         if line.trim() == "readyok" {
//             break;
//         }
//     }

//     stdin.write_all(format!("position fen {}\ngo depth 12\n", fen).as_bytes()).await?;

//     while let Some(line) = lines.next_line().await? {
//         if line.starts_with("bestmove") {
//             let parts: Vec<&str> = line.split_whitespace().collect();
//             return Ok(parts[1].to_string());
//         }
//     }

//     Err("No bestmove received".into())
// }

// // -------- Main --------

// fn get_root_path() -> &'static str {
//     env!("CARGO_MANIFEST_DIR")
// }

// #[actix_web::main]
// async fn main() -> std::io::Result<()> {
//     run_match().await.expect("Failed to run match");

//     println!("Server running on http://localhost:3000");

//     HttpServer::new(|| {
//         let cors = Cors::default()
//             // .allow_any_origin()
//             .allowed_origin("http://localhost:8001") // the frontend is running on port 8001
//             .allow_any_method()
//             .allow_any_header();

//         App::new().wrap(cors).service(get_meta).service(post_pgn).service(post_bestmove)
//     })
//     .bind(("127.0.0.1", 3000))?
//     .run()
//     .await
// }

// const PROJECT_ROOT: &str = env!("CARGO_MANIFEST_DIR");

// pub async fn run_match() -> Result<(), Box<dyn std::error::Error>> {
//     use async_process::Stdio;
//     use async_std::io::{BufReader, prelude::*};

//     let engine_1 = "BitboardX_v0.1.5".to_string();
//     let engine_2 = "BitboardX_v0.1.10".to_string();
//     let engine_1_name = format!("name={}", engine_1);
//     let engine_2_name = format!("name={}", engine_2);
//     let engine_1_cmd = format!("cmd={}/engines/{}.exe", PROJECT_ROOT, engine_1);
//     let engine_2_cmd = format!("cmd={}/engines/{}.exe", PROJECT_ROOT, engine_2);

//     let args = [
//         "-engine",
//         engine_1_name.as_str(),
//         engine_1_cmd.as_str(),
//         "-engine",
//         engine_2_name.as_str(),
//         engine_2_cmd.as_str(),
//         "-each",
//         "proto=uci",
//         "tc=inf",
//         "-rounds",
//         "10",
//         "-debug",
//         "all",
//         // "-pgnout",
//         // "out.pgn",
//     ];

//     let mut child = async_process::Command::new("cutechess-cli.exe")
//         .args(&args)
//         .stdout(Stdio::piped())
//         // .stderr(Stdio::piped())
//         .spawn()
//         .expect("Failed to execute process");

//     let stdout = child.stdout.take().expect("Failed to capture stdout");

//     let mut reader = BufReader::new(stdout).lines();
//     while let Some(line) = reader.next().await {
//         println!(">> {}", line?);
//     }

//     child.status().await?;
//     Ok(())
// }
