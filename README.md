# gomod-parser
[![Build Status](https://github.com/baz-scm/gomod-parser/workflows/PR/badge.svg)](https://github.com/baz-scm/gomod-parser/actions/workflows/pr.yml)
[![Crate](https://img.shields.io/crates/v/gomod-parser.svg)](https://crates.io/crates/gomod-parser)
[![MSRV](https://img.shields.io/crates/msrv/gomod-parser.svg)](https://blog.rust-lang.org/2022/09/22/Rust-1.64.0.html)

A simple `go.mod` file parser based on [winnow](https://crates.io/crates/winnow).

## Example
```rust
use gomod_parser::{GoMod, Module, ModuleDependency};
use std::str::FromStr;

let input = r#"
module github.com/example

go 1.21

require golang.org/x/net v0.20.0
"#;

let go_mod = GoMod::from_str(input).unwrap();

assert_eq!(go_mod.module, "github.com/example".to_string());
assert_eq!(go_mod.go, Some("1.21".to_string()));
assert_eq!(
    go_mod.require,
    vec![ModuleDependency {
        module: Module {
            module_path: "golang.org/x/net".to_string(),
            version: "v0.20.0".to_string()
        },
        indirect: false
    }]
);
```
