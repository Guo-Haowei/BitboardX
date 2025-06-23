use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, post, web};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
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

fn create_metadata_file(path: &PathBuf) {
    let stem = path.file_stem();
    if stem.is_none() {
        eprintln!("Failed to get file stem for path: {}", path.display());
        return;
    }
    // let stem = stem.unwrap();
    let meta_path = path.with_extension("json");
    eprintln!("Creating metadata file for: {}", meta_path.display());
    //     let meta_path = path.with_extension("json");
    //     let meta_json = serde_json::to_string(&metadata)?;
    //     fs::write(meta_path, meta_json)?;
    // }
    // match path.file_stem() {
    //     Ok(stem) => {
    //         let metadata = meta::MatchMeta {
    //             player1: "Player1".to_string(),
    //             player2: "Player2".to_string(),
    //             result: "1-0".to_string(),
    //             file: path.to_string_lossy().to_string(),
    //         };

    //         let meta_path = path.with_extension("json");
    //         let meta_json = serde_json::to_string(&metadata)?;
    //         fs::write(meta_path, meta_json)?;
    //     }
    //     None => {
    //     if let Some(stem_str) = stem.to_str() {
    //         println!("Base name without extension: {}", stem_str);
    //     }
    // } else {
    //     eprintln!("Failed to get file stem for path: {}", path.display());
    // }
    // Ok(())
}

pub fn process_pgn_files(dir: &Path) -> std::io::Result<()> {
    match fs::read_dir(dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                if path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map_or(false, |ext| ext.eq_ignore_ascii_case("pgn"))
                {
                    create_metadata_file(&path);
                }
            }

            Ok(())
        }
        Err(e) => Err(e),
    }
    // files
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Server running on http://localhost:3000");

    let root_path = env!("CARGO_MANIFEST_DIR");
    let root_path = Path::new(root_path);

    let match_dir = root_path.join("matches");
    let match_dir = match_dir.as_path();
    process_pgn_files(&match_dir).unwrap();

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
