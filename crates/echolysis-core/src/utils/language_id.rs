use std::path::Path;

pub fn get_language_id_by_file_extentsion(extension: &str) -> &'static str {
    match extension.to_lowercase().as_str() {
        // A
        "agda" => "agda",

        // B
        "bash" | "sh" | "zsh" => "bash",

        // C
        "c" | "h" => "c",
        "clj" | "cljs" | "cljc" => "clojure",
        "cmake" => "cmake",
        "cpp" | "hpp" | "cc" | "cxx" => "cpp",
        "cs" => "c_sharp",
        "css" => "css",

        // D
        "d" => "d",
        "dart" => "dart",
        "dockerfile" => "dockerfile",

        // E
        "el" | "elc" => "elisp",
        "ex" | "exs" => "elixir",
        "elm" => "elm",
        "erl" | "hrl" => "erlang",

        // F
        "fs" | "fsx" => "fsharp",
        "fish" => "fish",
        "f90" | "f95" | "f03" | "f08" => "fortran",

        // G
        "glsl" | "vert" | "frag" => "glsl",
        "go" => "go",
        "graphql" | "gql" => "graphql",

        // H
        "hack" | "hh" => "hack",
        "hs" | "lhs" => "haskell",
        "hcl" | "tf" => "hcl",
        "html" => "html",

        // J
        "java" => "java",
        "jl" => "julia",
        "js" | "jsx" => "javascript",
        "json" => "json",

        // K
        "kt" | "kts" => "kotlin",

        // L
        "lua" => "lua",

        // M
        "m" | "mat" => "matlab",
        "md" | "markdown" => "markdown",

        // N
        "nix" => "nix",

        // O
        "ml" | "mli" => "ocaml",

        // P
        "pas" | "pp" => "pascal",
        "perl" | "pl" | "pm" => "perl",
        "php" => "php",
        "proto" => "protobuf",
        "ps1" | "psm1" | "psd1" => "powershell",
        "py" => "python",

        // R
        "r" => "r",
        "rb" => "ruby",
        "rkt" => "racket",
        "rs" => "rust",

        // S
        "scala" => "scala",

        "scss" => "scss",
        "scm" => "scheme",
        "sql" => "sql",
        "svelte" => "svelte",
        "swift" => "swift",

        // T
        "toml" => "toml",
        "ts" | "tsx" => "typescript",

        // V
        "vue" => "vue",

        // Y
        "yaml" | "yml" => "yaml",

        // Z
        "zig" => "zig",

        _ => "",
    }
}

pub fn get_language_id_by_path(path: &Path) -> &'static str {
    if !path.is_file() {
        return "";
    }
    let extension = path
        .extension()
        .and_then(|x| x.to_str())
        .unwrap_or_default();

    get_language_id_by_file_extentsion(extension)
}
