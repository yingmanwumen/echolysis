use std::{path::PathBuf, str::FromStr, sync::Arc};

use echolysis_core::{engine::Engine, languages::SupportedLanguage};

#[global_allocator]
static GLOBAL: rpmalloc::RpMalloc = rpmalloc::RpMalloc;

pub fn main() {
    rayon::ThreadPoolBuilder::new()
        .num_threads(8)
        .build_global()
        .unwrap();

    let start = std::time::Instant::now();
    let paths = std::env::args()
        .skip(1)
        .filter_map(|path| PathBuf::from_str(&path).ok().map(Arc::new))
        .collect::<Vec<Arc<PathBuf>>>();
    let sources = paths
        .iter()
        .filter_map(|path| std::fs::read_to_string(path.as_path()).ok().map(Arc::new))
        .collect::<Vec<_>>();
    let sources = paths
        .into_iter()
        .zip(sources.iter().cloned())
        .collect::<Vec<_>>();
    let engine = Engine::new(SupportedLanguage::from_language_id("rust").unwrap());
    engine.insert_many(sources);
    let indexed = std::time::Instant::now();

    let detecting = std::time::Instant::now();
    let duplicates = engine.detect_duplicates(None);
    let dtected = std::time::Instant::now();

    for dup in &duplicates {
        println!("=======================================================");
        let len = dup.len();
        for (i, node) in dup.iter().enumerate() {
            let (start, end) = node.position_range();
            println!(
                "{}:{} {} lines long",
                node.path().to_str().unwrap_or_default(),
                start.row + 1,
                end.row - start.row + 1,
            );
            for _ in 0..start.column {
                print!(" ");
            }
            println!("{}", node.text());
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
