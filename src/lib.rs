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
//!         module: Module {
//!             module_path: "golang.org/x/net".to_string(),
//!             version: "v0.20.0".to_string()
//!         },
//!         indirect: false
//!     }]
//! );
//! ```

#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]

use crate::parser::{gomod, Directive};
use std::collections::HashMap;
use winnow::Parser;

mod combinator;
pub mod parser;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct GoMod {
    pub comment: Vec<String>,
    pub module: String,
    pub go: Option<String>,
    pub godebug: HashMap<String, String>,
    pub tool: Vec<String>,
    pub toolchain: Option<String>,
    pub require: Vec<ModuleDependency>,
    pub exclude: Vec<ModuleDependency>,
    pub replace: Vec<ModuleReplacement>,
    pub retract: Vec<ModuleRetract>,
}

impl std::str::FromStr for GoMod {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut res = Self::default();

        for directive in &mut gomod.parse(input).map_err(|e| e.to_string())? {
            match directive {
                Directive::Comment(d) => res.comment.push((**d).to_string()),
                Directive::Module(d) => res.module = (**d).to_string(),
                Directive::Go(d) => res.go = Some((**d).to_string()),
                Directive::GoDebug(d) => res.godebug.extend((*d).clone()),
                Directive::Tool(d) => res.tool.append(d),
                Directive::Toolchain(d) => res.toolchain = Some((**d).to_string()),
                Directive::Require(d) => res.require.append(d),
                Directive::Exclude(d) => res.exclude.append(d),
                Directive::Replace(d) => res.replace.append(d),
                Directive::Retract(d) => res.retract.append(d),
            }
        }

        Ok(res)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Module {
    pub module_path: String,
    pub version: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ModuleDependency {
    pub module: Module,
    pub indirect: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct ModuleReplacement {
    pub module_path: String,
    pub version: Option<String>,
    pub replacement: Replacement,
}

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
                module: Module {
                    module_path: "golang.org/x/net".to_string(),
                    version: "v0.20.0".to_string()
                },
                indirect: false
            }]
        );
        assert_eq!(
            go_mod.exclude,
            vec![ModuleDependency {
                module: Module {
                    module_path: "golang.org/x/net".to_string(),
                    version: "v0.19.1".to_string()
                },
                indirect: false
            }]
        );
        assert_eq!(
            go_mod.replace,
            vec![ModuleReplacement {
                module_path: "golang.org/x/net".to_string(),
                version: Some("v0.19.0".to_string()),
                replacement: Replacement::Module(Module {
                    module_path: "example.com/fork/net".to_string(),
                    version: "v0.19.1".to_string(),
                })
            }]
        );
        assert_eq!(
            go_mod.retract,
            vec![ModuleRetract::Single("v1.0.0".to_string())]
        );
        assert_eq!(go_mod.comment, vec!["Complete example".to_string()]);
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
                module: Module {
                    module_path: "golang.org/x/net".to_string(),
                    version: "v0.20.0".to_string()
                },
                indirect: false
            }]
        );
    }

    #[test]
    fn test_comments() {
        let input = indoc! {r#"
        module github.com/comments

        // 1st comment
        //2nd comment
          // 3rd comment
        "#};

        let go_mod = GoMod::from_str(input).unwrap();

        assert_eq!(go_mod.module, "github.com/comments".to_string());
        assert_eq!(
            go_mod.comment,
            vec![
                "1st comment".to_string(),
                "2nd comment".to_string(),
                "3rd comment".to_string(),
            ]
        );
    }
}
