# gomod-parser

[![Build Status](https://github.com/baz-scm/gomod-parser/workflows/PR/badge.svg)](https://github.com/baz-scm/gomod-parser/actions/workflows/pr.yml)
[![Coverage Status](https://coveralls.io/repos/github/baz-scm/gomod-parser/badge.svg?branch=main)](https://coveralls.io/github/baz-scm/gomod-parser?branch=main)
[![Crate](https://img.shields.io/crates/v/gomod-parser.svg)](https://crates.io/crates/gomod-parser)
[![MSRV](https://img.shields.io/crates/msrv/gomod-parser.svg)](https://blog.rust-lang.org/2022/11/03/Rust-1.65.0/)

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
        module: Module::new("golang.org/x/net", "v0.20.0"),
        indirect: false
    }]
);
```

## Positions

Every parsed module path and version carries its `Span` — a byte offset range
into the input — which makes the parser usable as a language server backend
(diagnostics, inlay hints, code actions):

```rust
use gomod_parser::GoMod;
use std::str::FromStr;

let input = "module github.com/example\n\nrequire golang.org/x/net v0.20.0\n";

let go_mod = GoMod::from_str(input).unwrap();
let dependency = &go_mod.require[0];

assert_eq!(&input[dependency.module.path_span.clone()], "golang.org/x/net");
assert_eq!(&input[dependency.module.version_span.clone()], "v0.20.0");
```

The `module` directive exposes its own `GoMod::module_span`, and `replace`
directives carry the same pair of spans on `ModuleReplacement` — with
`version_span` set to `None` for an unversioned source — plus the spans of the
nested `Module` when the destination is a module rather than a file path. Use
`Module::new` and `ModuleReplacement::new` to build values without spans.

Spans are ignored by `PartialEq`, so values parsed from different files still
compare equal by path and version.
