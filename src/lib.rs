#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]

use crate::parser::{gomod, Directive};
use winnow::Parser;

mod combinator;
pub mod parser;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct GoMod {
    pub comment: Vec<String>,
    pub module: Option<String>,
    pub go: Option<String>,
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
                Directive::Module(d) => res.module = Some((**d).to_string()),
                Directive::Go(d) => res.go = Some((**d).to_string()),
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
