# Echolysis

[![GitHub stars](https://img.shields.io/github/stars/yingmanwumen/echolysis)](https://github.com/yingmanwumen/echolysis/stargazers)
[![GitHub license](https://img.shields.io/github/license/yingmanwumen/echolysis)](https://github.com/yingmanwumen/echolysis/blob/main/LICENSE)
[![GitHub issues](https://img.shields.io/github/issues/yingmanwumen/echolysis)](https://github.com/yingmanwumen/echolysis/issues)
[![GitHub pull requests](https://img.shields.io/github/issues-pr/yingmanwumen/echolysis)](https://github.com/yingmanwumen/echolysis/pulls)

<!-- [![Crates.io](https://img.shields.io/crates/v/echolysis.svg)](https://crates.io/crates/echolysis) -->

<!-- [![docs.rs](https://docs.rs/echolysis/badge.svg)](https://docs.rs/echolysis) -->

> ⚠️ Note: Parts of this project's code and documentation are generated by AI, including this README.

Echolysis (`Echo` + `-lysis`) is an open-source duplicate code detection tool that aims to provide fast and accurate code duplication analysis across multiple programming languages.

## Why Echolysis?

While commercial IDEs like JetBrains products offer excellent duplicate code detection features, there's a gap in the open-source ecosystem. Existing open-source solutions often:

- Support only a limited set of programming languages
- Lack integration with modern development environments
- Have performance limitations with large codebases

Echolysis addresses these challenges by:

- Leveraging tree-sitter for robust and fast parsing across many languages
- Providing both CLI and Language Server Protocol (LSP) implementations (IN PROGRESS)
- Enabling easy integration with various editors and IDEs with the help of LSP (IN PROGRESS)
- Offering a language-agnostic approach that makes adding new language support straightforward (Currently only Rust and Python are supported)

## Features

- Fast and accurate duplicate code detection
- Support for multiple programming languages through tree-sitter (IN PROGRESS)
- CLI tool for command-line usage and CI/CD integration (IN PROGRESS)
- LSP server for real-time duplicate detection in editors
- Easy to extend with new language support

## Current Status

Echolysis currently supports:

- [x] Rust
- [x] Python
- [ ] C/C++
- [ ] ...

More languages will be added in the future.

## Components

- `echolysis-core`: Core duplicate detection engine
- `echolysis-cli`: Command line interface (TODO)
- `echolysis-lsp`: Language Server Protocol implementation

## License

[MIT License](LICENSE)

## Star History

[![Star History Chart](https://api.star-history.com/svg?repos=yingmanwumen/echolysis&type=Date)](https://star-history.com/#yingmanwumen/echolysis&Date)
