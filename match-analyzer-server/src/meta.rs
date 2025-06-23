use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize)]
pub struct MatchMeta {
    player1: String,
    player2: String,
    result: String,
    file: String,
}

struct GameMeta {
    white: String,
    black: String,
    result: String,
}

#[derive(Debug, Default)]
struct PlayerResult {
    win: u32,
    loss: u32,
    draw: u32,
}

fn create_metadata_file(path: &PathBuf) -> Result<(), String> {
    let stem = path.file_stem();
    eprintln!("Creating metadata file for: {}", path.display());
    if stem.is_none() {
        eprintln!("Failed to get file stem for path: {}", path.display());
        return Err(format!("File {} has no stem ", path.display()));
    }

    let meta_path = path.with_extension("json");
    // 1. If the metadata file already exists, check time modified matches the actual time modified
    if meta_path.exists() {
        std::fs::remove_file(&meta_path)
            .map_err(|e| format!("Failed to remove existing metadata file: {}", e))?;
    }

    eprintln!("Found result: {}", path.display());

    // 2. If it does not exist, create a new metadata file
    // @TODO: database
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read PGN file {}: {}", path.display(), e.to_string()))?;

    // Match White, Black, Result in expected order, allowing any whitespace
    let full_tag_re =
        Regex::new(r#"\[White\s+"([^"]+)"\]\s*\[Black\s+"([^"]+)"\]\s*\[Result\s+"([^"]+)"\]"#)
            .unwrap();

    let metas: Vec<GameMeta> = full_tag_re
        .captures_iter(&content)
        .map(|cap| GameMeta {
            white: cap[1].to_string(),
            black: cap[2].to_string(),
            result: cap[3].to_string(),
        })
        .collect();

    let stem = stem.unwrap();
    let stem = stem.to_str().ok_or("Failed to convert stem to string")?;
    let players = stem.split("-vs-").collect::<Vec<&str>>();
    if players.len() != 2
        || players[0].is_empty()
        || players[1].is_empty()
        || players[0] == players[1]
    {
        return Err(format!(
            "Invalid file name format for {}: expected 'player1-vs-player2'",
            path.display()
        ));
    }

    let mut player_results: HashMap<String, PlayerResult> = HashMap::new();
    player_results.insert(players[0].to_string(), PlayerResult::default());
    player_results.insert(players[1].to_string(), PlayerResult::default());

    for meta in metas.iter() {
        let white = meta.white.trim().to_string();
        let black = meta.black.trim().to_string();
        match meta.result.as_str() {
            "1-0" => {
                player_results.entry(white).and_modify(|r| r.win += 1);
                player_results.entry(black).and_modify(|r| r.loss += 1);
            }
            "0-1" => {
                player_results.entry(white).and_modify(|r| r.loss += 1);
                player_results.entry(black).and_modify(|r| r.win += 1);
            }
            "1/2-1/2" => {
                player_results.entry(white).and_modify(|r| r.draw += 1);
                player_results.entry(black).and_modify(|r| r.draw += 1);
            }
            _ => {
                panic!(
                    "Invalid result in PGN: {} for game between {} and {}",
                    meta.result, meta.white, meta.black
                );
            }
        }
    }

    println!("Player {} results: {:?}", players[0], player_results[players[0]]);
    println!("Player {} results: {:?}", players[1], player_results[players[1]]);
    println!("-------------------------");

    Ok(())
}

pub fn process_pgn_files(dir: &Path) -> Result<(), String> {
    match fs::read_dir(dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                if path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map_or(false, |ext| ext.eq_ignore_ascii_case("pgn"))
                {
                    create_metadata_file(&path)?;
                }
            }

            Ok(())
        }
        Err(e) => Err(e.to_string()),
    }
}

pub fn get_meta_impl() -> Vec<MatchMeta> {
    // let meta = fs::read_to_string("meta.json")
    //     .ok()
    //     .and_then(|s| serde_json::from_str::<Vec<MatchMeta>>(&s).ok())
    //     .unwrap_or_default();
    let meta = fs::read_to_string("meta.json")
        .ok()
        .and_then(|s| serde_json::from_str::<Vec<MatchMeta>>(&s).ok())
        .unwrap_or_default();
    // let meta = "{ a: 1 }";

    meta
}
