//! A simple `go.mod` file parser
//!
//! # Example
//!
//! ```rust
//! use gomod_parser::{GoMod, Module, ModuleDependency};
//! use std::str::FromStr;
//!
//! let input = r#"
//! module github.com/example
//!
//! go 1.21
//!
//! require golang.org/x/net v0.20.0
//! "#;
//!
//! let go_mod = GoMod::from_str(input).unwrap();
//!
//! assert_eq!(go_mod.module, "github.com/example".to_string());
//! assert_eq!(go_mod.go, Some("1.21".to_string()));
//! assert_eq!(
//!     go_mod.require,
//!     vec![ModuleDependency {
//!         module: Module::new("golang.org/x/net", "v0.20.0"),
//!         indirect: false
//!     }]
//! );
//! ```
//!
//! # Positions
//!
//! Every parsed module path and version carries its [`Span`] — a byte offset
//! range into the input — which makes the parser usable as a language server
//! backend. The same holds for [`GoMod::module_span`] and for both sides of a
//! `replace` directive. Spans do not take part in [`PartialEq`], so values
//! parsed from different offsets still compare equal by path and version:
//!
//! ```rust
//! use gomod_parser::GoMod;
//! use std::str::FromStr;
//!
//! let input = "module github.com/example\n\nrequire golang.org/x/net v0.20.0\n";
//!
//! let go_mod = GoMod::from_str(input).unwrap();
//! let dependency = &go_mod.require[0];
//!
//! assert_eq!(&input[dependency.module.path_span.clone()], "golang.org/x/net");
//! assert_eq!(&input[dependency.module.version_span.clone()], "v0.20.0");
//! ```

#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]

use crate::parser::{gomod, Directive};
use std::collections::HashMap;
use std::ops::Range;
use winnow::stream::LocatingSlice;
use winnow::Parser;

mod combinator;
pub mod parser;

#[derive(Debug, Default)]
pub struct GoMod {
    pub comment: Vec<String>,
    pub module: String,
    pub module_span: Span,
    pub go: Option<String>,
    pub godebug: HashMap<String, String>,
    pub tool: Vec<String>,
    pub toolchain: Option<String>,
    pub require: Vec<ModuleDependency>,
    pub exclude: Vec<ModuleDependency>,
    pub replace: Vec<ModuleReplacement>,
    pub retract: Vec<ModuleRetract>,
    pub ignore: Vec<String>,
}

impl PartialEq for GoMod {
    fn eq(&self, other: &Self) -> bool {
        self.comment == other.comment
            && self.module == other.module
            && self.go == other.go
            && self.godebug == other.godebug
            && self.tool == other.tool
            && self.toolchain == other.toolchain
            && self.require == other.require
            && self.exclude == other.exclude
            && self.replace == other.replace
            && self.retract == other.retract
            && self.ignore == other.ignore
    }
}

impl Eq for GoMod {}

impl std::str::FromStr for GoMod {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut res = Self::default();

        for directive in &mut gomod
            .parse(LocatingSlice::new(input))
            .map_err(|e| e.to_string())?
        {
            match directive {
                Directive::Comment(d) => res.comment.push((**d).to_string()),
                Directive::Module(d, span) => {
                    res.module = (**d).to_string();
                    res.module_span = span.clone();
                }
                Directive::Go(d) => res.go = Some((**d).to_string()),
                Directive::GoDebug(d) => res.godebug.extend((*d).clone()),
                Directive::Tool(d) => res.tool.append(d),
                Directive::Toolchain(d) => res.toolchain = Some((**d).to_string()),
                Directive::Require(d) => res.require.append(d),
                Directive::Exclude(d) => res.exclude.append(d),
                Directive::Replace(d) => res.replace.append(d),
                Directive::Retract(d) => res.retract.append(d),
                Directive::Ignore(d) => res.ignore.append(d),
            }
        }

        Ok(res)
    }
}

pub type Span = Range<usize>;

#[derive(Debug)]
pub struct Module {
    pub module_path: String,
    pub version: String,
    pub path_span: Span,
    pub version_span: Span,
}

impl Module {
    #[must_use]
    pub fn new(module_path: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            module_path: module_path.into(),
            version: version.into(),
            path_span: 0..0,
            version_span: 0..0,
        }
    }
}

impl PartialEq for Module {
    fn eq(&self, other: &Self) -> bool {
        self.module_path == other.module_path && self.version == other.version
    }
}

impl Eq for Module {}

#[derive(Debug, PartialEq, Eq)]
pub struct ModuleDependency {
    pub module: Module,
    pub indirect: bool,
}

#[derive(Debug)]
pub struct ModuleReplacement {
    pub module_path: String,
    pub version: Option<String>,
    pub path_span: Span,
    pub version_span: Option<Span>,
    pub replacement: Replacement,
}

impl ModuleReplacement {
    #[must_use]
    pub fn new(
        module_path: impl Into<String>,
        version: Option<String>,
        replacement: Replacement,
    ) -> Self {
        Self {
            module_path: module_path.into(),
            version,
            path_span: 0..0,
            version_span: None,
            replacement,
        }
    }
}

impl PartialEq for ModuleReplacement {
    fn eq(&self, other: &Self) -> bool {
        self.module_path == other.module_path
            && self.version == other.version
            && self.replacement == other.replacement
    }
}

impl Eq for ModuleReplacement {}

#[derive(Debug, PartialEq, Eq)]
pub enum Replacement {
    FilePath(String),
    Module(Module),
}

#[derive(Debug, PartialEq, Eq)]
pub enum ModuleRetract {
    Single(String),
    Range(String, String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;
    use std::str::FromStr;

    #[test]
    fn test_parse_complete() {
        let input = indoc! {r#"
        // Complete example

        module github.com/complete

        go 1.21

        toolchain go1.21.1

        require golang.org/x/net v0.20.0

        exclude golang.org/x/net v0.19.1

        replace golang.org/x/net v0.19.0 => example.com/fork/net v0.19.1

        retract v1.0.0
        "#};

        let go_mod = GoMod::from_str(input).unwrap();

        assert_eq!(go_mod.module, "github.com/complete".to_string());
        assert_eq!(go_mod.go, Some("1.21".to_string()));
        assert_eq!(go_mod.toolchain, Some("go1.21.1".to_string()));
        assert_eq!(
            go_mod.require,
            vec![ModuleDependency {
                module: Module::new("golang.org/x/net", "v0.20.0"),
                indirect: false
            }]
        );
        assert_eq!(
            go_mod.exclude,
            vec![ModuleDependency {
                module: Module::new("golang.org/x/net", "v0.19.1"),
                indirect: false
            }]
        );
        assert_eq!(
            go_mod.replace,
            vec![ModuleReplacement::new(
                "golang.org/x/net",
                Some("v0.19.0".to_string()),
                Replacement::Module(Module::new("example.com/fork/net", "v0.19.1"))
            )]
        );
        assert_eq!(
            go_mod.retract,
            vec![ModuleRetract::Single("v1.0.0".to_string())]
        );
        assert_eq!(go_mod.comment, vec!["Complete example".to_string()]);
    }

    #[test]
    fn test_module_span() {
        let input = indoc! {r#"
        // leading comment

        module github.com/spans

        go 1.24
        "#};

        let go_mod = GoMod::from_str(input).unwrap();

        assert_eq!(go_mod.module, "github.com/spans".to_string());
        assert_eq!(&input[go_mod.module_span.clone()], "github.com/spans");
    }

    #[test]
    fn test_module_trailing_text() {
        for (input, expected) in [
            ("module github.com/spans // some comment\n", 7..23),
            ("module github.com/spans   \n", 7..23),
            ("module github.com/spans\r\n", 7..23),
            ("module github.com/spans", 7..23),
        ] {
            let go_mod = GoMod::from_str(input).unwrap();

            assert_eq!(
                go_mod.module,
                "github.com/spans".to_string(),
                "input: {input:?}"
            );
            assert_eq!(go_mod.module_span, expected, "input: {input:?}");
            assert_eq!(&input[go_mod.module_span.clone()], "github.com/spans");
        }
    }

    #[test]
    fn test_require_spans() {
        let input = indoc! {r#"
        module github.com/spans

        require (
            golang.org/x/net v0.20.0
            golang.org/x/sys v0.16.0 // indirect
        )
        "#};

        let go_mod = GoMod::from_str(input).unwrap();

        let net = &go_mod.require[0].module;
        assert_eq!(&input[net.path_span.clone()], "golang.org/x/net");
        assert_eq!(&input[net.version_span.clone()], "v0.20.0");

        let sys = &go_mod.require[1].module;
        assert_eq!(&input[sys.path_span.clone()], "golang.org/x/sys");
        assert_eq!(&input[sys.version_span.clone()], "v0.16.0");

        assert!(net.version_span.end < sys.path_span.start);
    }

    #[test]
    fn test_exclude_spans() {
        let input = indoc! {r#"
        module github.com/spans

        exclude golang.org/x/net v0.19.1
        "#};

        let go_mod = GoMod::from_str(input).unwrap();

        let module = &go_mod.exclude[0].module;
        assert_eq!(&input[module.path_span.clone()], "golang.org/x/net");
        assert_eq!(&input[module.version_span.clone()], "v0.19.1");
    }

    #[test]
    fn test_replace_spans() {
        let input = indoc! {r#"
        module github.com/spans

        replace (
            golang.org/x/net v0.19.0 => example.com/fork/net v0.19.1
            golang.org/x/sys => ../sys
        )
        "#};

        let go_mod = GoMod::from_str(input).unwrap();

        let versioned = &go_mod.replace[0];
        assert_eq!(&input[versioned.path_span.clone()], "golang.org/x/net");
        assert_eq!(&input[versioned.version_span.clone().unwrap()], "v0.19.0");
        match &versioned.replacement {
            Replacement::Module(module) => {
                assert_eq!(&input[module.path_span.clone()], "example.com/fork/net");
                assert_eq!(&input[module.version_span.clone()], "v0.19.1");
            }
            Replacement::FilePath(path) => panic!("unexpected file path replacement: {path}"),
        }

        let unversioned = &go_mod.replace[1];
        assert_eq!(&input[unversioned.path_span.clone()], "golang.org/x/sys");
        assert_eq!(unversioned.version_span, None);
    }

    #[test]
    fn test_spans_ignored_by_eq() {
        let input = indoc! {r#"
        module github.com/spans

        require golang.org/x/net v0.20.0
        "#};

        let go_mod = GoMod::from_str(input).unwrap();

        assert_ne!(go_mod.require[0].module.version_span, 0..0);
        assert_eq!(
            go_mod.require[0].module,
            Module::new("golang.org/x/net", "v0.20.0")
        );
    }

    #[test]
    fn test_invalid_content() {
        let input = indoc! {r#"
        modulegithub.com/no-space
        "#};

        let go_mod = GoMod::from_str(input);

        assert!(go_mod.is_err());
    }

    #[test]
    fn test_no_line_ending_after_module() {
        let input = indoc! {r#"
        module github.com/no-line-ending"#};

        let go_mod = GoMod::from_str(input).unwrap();

        assert_eq!(go_mod.module, "github.com/no-line-ending".to_string());
    }

    #[test]
    fn test_no_line_ending_after_go() {
        let input = indoc! {r#"
        module github.com/no-line-ending

        go 1.24"#};

        let go_mod = GoMod::from_str(input).unwrap();

        assert_eq!(go_mod.go, Some("1.24".to_string()));
    }

    #[test]
    fn test_no_line_ending_after_godebug() {
        let input = indoc! {r#"
        module github.com/no-line-ending

        godebug (
            default=go1.21
            panicnil=1
        )"#};

        let go_mod = GoMod::from_str(input).unwrap();

        assert_eq!(
            go_mod.godebug,
            HashMap::from([
                ("default".to_string(), "go1.21".to_string()),
                ("panicnil".to_string(), "1".to_string())
            ])
        );
    }

    #[test]
    fn test_no_line_ending_after_tool() {
        let input = indoc! {r#"
        module github.com/no-line-ending

        tool example.com/mymodule/cmd/mytool1"#};

        let go_mod = GoMod::from_str(input).unwrap();

        assert_eq!(
            go_mod.tool,
            vec!["example.com/mymodule/cmd/mytool1".to_string()]
        );
    }

    #[test]
    fn test_no_line_ending_after_toolchain() {
        let input = indoc! {r#"
        module github.com/no-line-ending

        toolchain go1.21.1"#};

        let go_mod = GoMod::from_str(input).unwrap();

        assert_eq!(go_mod.toolchain, Some("go1.21.1".to_string()));
    }

    #[test]
    fn test_no_line_ending_after_require() {
        let input = indoc! {r#"
        module github.com/no-line-ending

        require (
            golang.org/x/net v0.20.0
        )"#};

        let go_mod = GoMod::from_str(input).unwrap();

        assert_eq!(
            go_mod.require,
            vec![ModuleDependency {
                module: Module::new("golang.org/x/net", "v0.20.0"),
                indirect: false
            }]
        );
    }

    #[test]
    fn test_ignore_single() {
        let input = indoc! {r#"
        module github.com/ignore-single

        go 1.24

        ignore ./testdata
        "#};

        let go_mod = GoMod::from_str(input).unwrap();

        assert_eq!(go_mod.ignore, vec!["./testdata".to_string()]);
    }

    #[test]
    fn test_ignore_multi() {
        let input = indoc! {r#"
        module github.com/ignore-multi

        go 1.24

        ignore (
            ./testdata
            ./vendor/temp
            ./node_modules
        )
        "#};

        let go_mod = GoMod::from_str(input).unwrap();

        assert_eq!(
            go_mod.ignore,
            vec![
                "./testdata".to_string(),
                "./vendor/temp".to_string(),
                "./node_modules".to_string(),
            ]
        );
    }

    #[test]
    fn test_ignore_repeated_singles() {
        let input = indoc! {r#"
        module github.com/ignore-repeated

        go 1.24

        ignore ./testdata
        ignore ./vendor/temp
        "#};

        let go_mod = GoMod::from_str(input).unwrap();

        assert_eq!(
            go_mod.ignore,
            vec!["./testdata".to_string(), "./vendor/temp".to_string()]
        );
    }

    #[test]
    fn test_no_line_ending_after_ignore() {
        let input = indoc! {r#"
        module github.com/no-line-ending

        ignore (
            ./testdata
        )"#};

        let go_mod = GoMod::from_str(input).unwrap();

        assert_eq!(go_mod.ignore, vec!["./testdata".to_string()]);
    }

    #[test]
    fn test_comments() {
        let input = indoc! {r#"
        module github.com/comments

        // 1st comment
        //2nd comment
          // 3rd comment
          //4th comment"#};

        let go_mod = GoMod::from_str(input).unwrap();

        assert_eq!(go_mod.module, "github.com/comments".to_string());
        assert_eq!(
            go_mod.comment,
            vec![
                "1st comment".to_string(),
                "2nd comment".to_string(),
                "3rd comment".to_string(),
                "4th comment".to_string(),
            ]
        );
    }

    #[test]
    fn test_directive_inline_comments() {
        let input = indoc! {r#"
        module github.com/inline-comments

        exclude (
            // 1st comment
              //2nd comment
            example.com/dependency v1.2.3
              // 3rd comment
            //4th comment
        )"#};

        let go_mod = GoMod::from_str(input).unwrap();

        assert_eq!(go_mod.module, "github.com/inline-comments".to_string());
        assert_eq!(
            go_mod.exclude,
            vec![ModuleDependency {
                module: Module::new("example.com/dependency", "v1.2.3"),
                indirect: false,
            }]
        );
        assert!(go_mod.comment.is_empty());
    }
}
