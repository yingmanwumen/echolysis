use std::path::Path;

/// Converts a file extension to a standardized language identifier, primarily supporting
/// tree-sitter compatible languages.
///
/// This function maps file extensions to their corresponding language identifiers that are
/// commonly used with tree-sitter parsers. It handles a wide range of programming languages
/// and file formats.
///
/// # Arguments
///
/// * `path` - A Path reference to the file
///
/// # Returns
///
/// * A static string representing the language identifier
/// * Returns an empty string ("") if:
///   - The path is not a file
///   - The extension is not recognized
///   - The path has no extension or filename
pub fn file_extension_to_language_id(path: &Path) -> &'static str {
    if !path.is_file() {
        return "";
    }
    let extension = path
        .extension()
        .unwrap_or_else(|| path.file_name().unwrap_or_default())
        .to_str()
        .unwrap_or_default()
        .to_lowercase();
    match extension.as_str() {
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
