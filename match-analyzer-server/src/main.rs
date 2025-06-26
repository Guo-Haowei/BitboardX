use futures::{SinkExt, StreamExt, stream::SplitSink};
use regex::Regex;
use serde::Serialize;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::{WebSocketStream, accept_async};

const PROJECT_ROOT: &'static str = env!("CARGO_MANIFEST_DIR");

type SplitSinkStream = SplitSink<WebSocketStream<TcpStream>, Message>;

#[derive(Serialize)]
struct BestMoveMessage {
    r#type: &'static str,
    bestmove: String,
}

#[derive(Serialize)]
struct NewGameMessage {
    r#type: &'static str,
    game: i32,
    total: i32,
    white: String,
    black: String,
}

async fn run_match(
    ws_sink: &mut SplitSinkStream,
    engine_1: &str,
    engine_2: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use async_process::Stdio;
    use async_std::io::{BufReader, prelude::*};

    let engine_1_name = format!("name={}", engine_1);
    let engine_2_name = format!("name={}", engine_2);

    let engine_1_path = format!("{}\\engines\\{}.exe", PROJECT_ROOT, engine_1);
    let engine_2_path = format!("{}\\engines\\{}.exe", PROJECT_ROOT, engine_2);

    if !std::path::Path::new(&engine_1_path).exists() {
        return Err(Box::from(format!("Engine 1 not found: {}", engine_1_path)));
    }
    if !std::path::Path::new(&engine_2_path).exists() {
        return Err(Box::from(format!("Engine 2 not found: {}", engine_2_path)));
    }

    let engine_1_cmd = format!("cmd={}", engine_1_path);
    let engine_2_cmd = format!("cmd={}", engine_2_path);

    let args = [
        "-engine",
        engine_1_name.as_str(),
        engine_1_cmd.as_str(),
        "-engine",
        engine_2_name.as_str(),
        engine_2_cmd.as_str(),
        "-each",
        "proto=uci",
        "tc=inf",
        "-rounds",
        "10",
        "-debug",
        "all",
        "-pgnout",
        "out.pgn",
    ];

    let mut child = async_process::Command::new("cutechess-cli.exe")
        .args(&args)
        .stdout(Stdio::piped())
        // .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to execute process");

    let stdout = child.stdout.take().expect("Failed to capture stdout");

    let mut reader = BufReader::new(stdout).lines();
    while let Some(line) = reader.next().await {
        // Starting match between BitboardX_v0.1.5 and BitboardX_v0.1.10
        let line = line?;

        let re = Regex::new(r#"Started game (\d+) of (\d+) \(([^)]+) vs ([^)]+)\)"#).unwrap();
        if let Some(caps) = re.captures(&line) {
            let new_game_message = NewGameMessage {
                r#type: "newgame",
                game: caps.get(0).unwrap().as_str().parse().unwrap_or(0),
                total: caps.get(1).unwrap().as_str().parse().unwrap_or(0),
                white: caps.get(2).unwrap().as_str().to_string(),
                black: caps.get(3).unwrap().as_str().to_string(),
            };
            ws_sink.send(Message::Text(serde_json::to_string(&new_game_message)?)).await?;
            continue;
        }

        let re = Regex::new(r"bestmove\s+(\S+)").unwrap();
        if let Some(caps) = re.captures(&line) {
            if let Some(bestmove) = caps.get(1) {
                let bestmove_msg =
                    BestMoveMessage { r#type: "bestmove", bestmove: bestmove.as_str().to_string() };
                ws_sink.send(Message::Text(serde_json::to_string(&bestmove_msg)?)).await?;
            }
            continue;
        }

        if line.starts_with("Finished game") {
            ws_sink.send(Message::Text(line)).await?;
            continue;
        }

        // println!("{}", line);
    }

    child.status().await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:3000").await.unwrap();
    println!("Listening on ws://127.0.0.1:3000");

    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(async move {
            let ws_stream = accept_async(stream).await.unwrap();
            let (mut ws_sink, mut ws_stream) = ws_stream.split();

            'messageloop: while let Some(Ok(msg)) = ws_stream.next().await {
                if let Message::Text(text) = msg {
                    let mut err_msg = String::new();
                    let parts: Vec<_> = text.split(':').collect();
                    loop {
                        if text.starts_with("match:") {
                            if parts.len() != 3 {
                                err_msg =
                                    "Error: match command requires two player names".to_string();
                                break;
                            }

                            println!("Starting match between {} and {}", parts[1], parts[2]);

                            let result = run_match(&mut ws_sink, parts[1], parts[2]).await;
                            if let Err(e) = result {
                                err_msg = format!("Error running match: {}", e);
                            }

                            break;
                        }

                        // if manifest, send manifest

                        err_msg = format!("Error: unknown command '{}'", text);
                        break;
                    }
                    if !err_msg.is_empty() && ws_sink.send(Message::Text(err_msg)).await.is_err() {
                        println!("Client disconnected before receiving error message");
                        break 'messageloop;
                    }
                } else if msg.is_close() {
                    println!("Client closed connection");
                    break;
                }
            }
        });
    }
}
