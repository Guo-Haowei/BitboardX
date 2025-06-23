use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize)]
pub struct MatchMeta {
    player1: String,
    player2: String,
    result: String,
    file: String,
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
