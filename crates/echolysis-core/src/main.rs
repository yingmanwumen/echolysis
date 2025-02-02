use std::sync::Arc;

use ahash::AHashMap;
use echolysis_core::{languages::SupportedLanguage, utils::tree::NodeExt, Engine};

#[global_allocator]
static GLOBAL: rpmalloc::RpMalloc = rpmalloc::RpMalloc;

pub fn main() {
    rayon::ThreadPoolBuilder::new()
        .num_threads(8)
        .build_global()
        .unwrap();

    let start = std::time::Instant::now();
    let paths = std::env::args().skip(1).collect::<Vec<String>>();
    let sources = paths
        .iter()
        .map(|path| std::fs::read_to_string(path).unwrap())
        .collect::<Vec<_>>();
    let sources = paths
        .into_iter()
        .zip(&sources)
        .map(|(path, source)| (Arc::new(path), source.as_str()))
        .collect::<AHashMap<_, _>>();
    let engine = Engine::new(
        SupportedLanguage::from_language_id("rust").unwrap(),
        Some(sources.clone()),
    );
    let indexed = std::time::Instant::now();

    let detecting = std::time::Instant::now();
    let duplicates = engine.detect_duplicates();
    let dtected = std::time::Instant::now();

    let language = engine.language();

    for dup in &duplicates {
        println!("=======================================================");
        let len = dup.len();
        for (i, &id) in dup.iter().enumerate() {
            let node = engine.get_node_by_id(id).unwrap();
            let path = engine.get_path_by_id(id).unwrap();
            let start = node.start_position().row + 1;
            let end = node.end_position().row + 1;
            println!(
                "{}:{} {} lines long, cognitive complexity: {}",
                path,
                start,
                end - start + 1,
                language.cognitive_complexity(node)
            );
            for _ in 0..node.start_position().column {
                print!(" ");
            }
            println!("{}", node.text(sources.get(&path).unwrap().as_bytes()));
            if i != len - 1 {
                println!("-------------------------------------------------------");
            }
        }
    }
    println!("#######################################################");
    println!("duplicates: {}", duplicates.len());
    println!(
        "indexing cost: {} ms",
        indexed.duration_since(start).as_millis()
    );
    println!(
        "detecting cost: {} ms",
        dtected.duration_since(detecting).as_millis()
    );
}
