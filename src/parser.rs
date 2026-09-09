use crate::combinator::not_whitespace;
use crate::{Module, ModuleDependency, ModuleReplacement, ModuleRetract, Replacement, Span};
use std::collections::HashMap;
use winnow::ascii::{multispace0, multispace1, space0, space1};
use winnow::combinator::{alt, fail, not, opt, peek, preceded, repeat, terminated};
use winnow::error::ContextError;
use winnow::stream::{AsChar, LocatingSlice};
use winnow::token::{any, take_till, take_while};
use winnow::{dispatch, Parser, Result};

const WHITESPACES: [char; 4] = [' ', '\t', '\r', '\n'];
const CRLF: [char; 2] = ['\r', '\n'];

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Directive<'a> {
    Comment(&'a str),
    Module(&'a str, Span),
    Go(&'a str),
    GoDebug(HashMap<String, String>),
    Tool(Vec<String>),
    Toolchain(&'a str),
    Require(Vec<ModuleDependency>),
    Exclude(Vec<ModuleDependency>),
    Replace(Vec<ModuleReplacement>),
    Retract(Vec<ModuleRetract>),
    Ignore(Vec<String>),
}

pub(crate) fn gomod<'a>(input: &mut LocatingSlice<&'a str>) -> Result<Vec<Directive<'a>>> {
    repeat(0.., |i: &mut LocatingSlice<&'a str>| {
        // check for comments first
        comment.parse_next(i).or_else(|_| directive.parse_next(i))
    })
    .parse_next(input)
}

fn directive<'a>(input: &mut LocatingSlice<&'a str>) -> Result<Directive<'a>> {
    let _ = take_while(0.., CRLF).parse_next(input)?;
    dispatch!(peek(not_whitespace);
        "module" => module,
        "go" => go,
        "godebug" => godebug,
        "tool" => tool,
        "toolchain" => toolchain,
        "require" => require,
        "exclude" => exclude,
        "replace" => replace,
        "retract" => retract,
        "ignore" => ignore,
        _ => fail,
    )
    .parse_next(input)
}

fn comment<'a>(input: &mut LocatingSlice<&'a str>) -> Result<Directive<'a>> {
    let res = preceded((opt(space0), "//", opt(space0)), take_till(0.., CRLF)).parse_next(input)?;
    let _ = take_while(0.., CRLF).parse_next(input)?;

    Ok(Directive::Comment(res))
}

fn multi<'a, T, P>(input: &mut LocatingSlice<&'a str>, mut entry: P) -> Result<Vec<T>>
where
    P: Parser<LocatingSlice<&'a str>, Vec<T>, ContextError>,
{
    ("(", multispace1).parse_next(input)?;

    let entries: Vec<Option<Vec<T>>> = repeat(
        1..,
        terminated(
            alt((
                comment.map(|_| None), // skips any comments inside a multiline directive
                |input: &mut LocatingSlice<&'a str>| entry.parse_next(input).map(Some),
            )),
            multispace0,
        ),
    )
    .parse_next(input)?;

    (")", multispace0).parse_next(input)?;

    Ok(entries.into_iter().flatten().flatten().collect())
}

fn module<'a>(input: &mut LocatingSlice<&'a str>) -> Result<Directive<'a>> {
    let (res, span) =
        preceded(("module", space1), take_till(1.., WHITESPACES).with_span()).parse_next(input)?;

    // remove any comments added to the same line
    let _ = (space0, opt(comment)).parse_next(input)?;
    let _ = take_while(0.., CRLF).parse_next(input)?;

    Ok(Directive::Module(res, span))
}

fn go<'a>(input: &mut LocatingSlice<&'a str>) -> Result<Directive<'a>> {
    let res = preceded(("go", space1), take_till(1.., CRLF)).parse_next(input)?;
    let _ = take_while(0.., CRLF).parse_next(input)?;

    Ok(Directive::Go(res))
}

fn godebug<'a>(input: &mut LocatingSlice<&'a str>) -> Result<Directive<'a>> {
    let res = preceded(
        ("godebug", space1),
        dispatch! {peek(any);
            '(' => godebug_multi,
            _ => godebug_single,
        },
    )
    .parse_next(input)?;
    let _ = take_while(0.., CRLF).parse_next(input)?;

    Ok(Directive::GoDebug(HashMap::from_iter(res)))
}

fn godebug_single(input: &mut LocatingSlice<&str>) -> Result<Vec<(String, String)>> {
    // terminate, if `)` is found
    peek(not(')')).parse_next(input)?;

    let (key, _, value) =
        (take_till(1.., '='), '=', take_till(1.., WHITESPACES)).parse_next(input)?;

    Ok(vec![(key.into(), value.into())])
}

fn godebug_multi(input: &mut LocatingSlice<&str>) -> Result<Vec<(String, String)>> {
    multi(input, godebug_single)
}

fn tool<'a>(input: &mut LocatingSlice<&'a str>) -> Result<Directive<'a>> {
    let res = preceded(
        ("tool", space1),
        dispatch! {peek(any);
            '(' => tool_multi,
            _ => tool_single,
        },
    )
    .parse_next(input)?;
    let _ = take_while(0.., CRLF).parse_next(input)?;

    Ok(Directive::Tool(res))
}

fn tool_single(input: &mut LocatingSlice<&str>) -> Result<Vec<String>> {
    // terminate, if `)` is found
    peek(not(')')).parse_next(input)?;

    let value = take_till(1.., WHITESPACES).parse_next(input)?;

    // remove any comments added to the same line
    let _ = opt(comment).parse_next(input)?;

    Ok(vec![value.into()])
}

fn tool_multi(input: &mut LocatingSlice<&str>) -> Result<Vec<String>> {
    multi(input, tool_single)
}

fn toolchain<'a>(input: &mut LocatingSlice<&'a str>) -> Result<Directive<'a>> {
    let res = preceded(("toolchain", space1), take_till(1.., CRLF)).parse_next(input)?;
    let _ = take_while(0.., CRLF).parse_next(input)?;

    Ok(Directive::Toolchain(res))
}

fn require<'a>(input: &mut LocatingSlice<&'a str>) -> Result<Directive<'a>> {
    let res = preceded(
        ("require", space1),
        dispatch! {peek(any);
            '(' => require_multi,
            _ => require_single,
        },
    )
    .parse_next(input)?;
    let _ = take_while(0.., CRLF).parse_next(input)?;

    Ok(Directive::Require(res))
}

fn require_single(input: &mut LocatingSlice<&str>) -> Result<Vec<ModuleDependency>> {
    // terminate, if `)` is found
    peek(not(')')).parse_next(input)?;

    let ((module_path, path_span), _, (version, version_span)) = (
        take_till(1.., AsChar::is_space).with_span(),
        space1,
        take_till(1.., WHITESPACES).with_span(),
    )
        .parse_next(input)?;

    let indirect = opt(comment).parse_next(input)? == Some(Directive::Comment("indirect"));

    Ok(vec![ModuleDependency {
        module: Module {
            module_path: module_path.to_string(),
            version: version.to_string(),
            path_span,
            version_span,
        },
        indirect,
    }])
}

fn require_multi(input: &mut LocatingSlice<&str>) -> Result<Vec<ModuleDependency>> {
    multi(input, require_single)
}

fn exclude<'a>(input: &mut LocatingSlice<&'a str>) -> Result<Directive<'a>> {
    let res = preceded(
        ("exclude", space1),
        dispatch! {peek(any);
            '(' => require_multi,
            _ => require_single,
        },
    )
    .parse_next(input)?;
    let _ = take_while(0.., CRLF).parse_next(input)?;

    Ok(Directive::Exclude(res))
}

fn replace<'a>(input: &mut LocatingSlice<&'a str>) -> Result<Directive<'a>> {
    let res = preceded(
        ("replace", space1),
        dispatch! {peek(any);
            '(' => replace_multi,
            _ => replace_single,
        },
    )
    .parse_next(input)?;
    let _ = take_while(0.., CRLF).parse_next(input)?;

    Ok(Directive::Replace(res))
}

fn replace_single(input: &mut LocatingSlice<&str>) -> Result<Vec<ModuleReplacement>> {
    // terminate, if `)` is found
    peek(not(')')).parse_next(input)?;

    let ((src_path, src_path_span), src_version) = (
        terminated(take_till(1.., AsChar::is_space).with_span(), space1),
        opt(terminated(
            preceded(
                peek(not("=>")),
                take_till(1.., AsChar::is_space).with_span(),
            ),
            space1,
        )),
    )
        .parse_next(input)?;
    let _ = ("=>", space1).parse_next(input)?;
    let ((dest_path, dest_path_span), dest_version) = (
        terminated(take_till(1.., WHITESPACES).with_span(), space0),
        opt(terminated(
            take_till(1.., WHITESPACES).with_span(),
            multispace1,
        )),
    )
        .parse_next(input)?;

    let replacement = dest_version.map_or_else(
        || Replacement::FilePath(dest_path.to_string()),
        |(version, version_span)| {
            Replacement::Module(Module {
                module_path: dest_path.to_string(),
                version: version.to_string(),
                path_span: dest_path_span,
                version_span,
            })
        },
    );

    let (src_version, src_version_span) = src_version.map_or((None, None), |(version, span)| {
        (Some(version.to_string()), Some(span))
    });

    Ok(vec![ModuleReplacement {
        module_path: src_path.to_string(),
        version: src_version,
        path_span: src_path_span,
        version_span: src_version_span,
        replacement,
    }])
}

fn replace_multi(input: &mut LocatingSlice<&str>) -> Result<Vec<ModuleReplacement>> {
    multi(input, replace_single)
}

fn retract<'a>(input: &mut LocatingSlice<&'a str>) -> Result<Directive<'a>> {
    let res = preceded(
        ("retract", space1),
        dispatch! {peek(any);
            '(' => retract_multi,
            _ => retract_single,
        },
    )
    .parse_next(input)?;
    let _ = take_while(0.., CRLF).parse_next(input)?;

    Ok(Directive::Retract(res))
}

fn retract_single(input: &mut LocatingSlice<&str>) -> Result<Vec<ModuleRetract>> {
    // terminate, if `)` is found
    peek(not(')')).parse_next(input)?;

    let res = dispatch! {peek(any);
        '[' => version_range,
        _ => version_single,
    }
    .parse_next(input)?;

    // remove any comments added to the same line
    let _ = opt(comment).parse_next(input)?;

    Ok(vec![res])
}

fn version_range(input: &mut LocatingSlice<&str>) -> Result<ModuleRetract> {
    let lower_bound = preceded('[', take_till(1.., |c| c == ',' || c == ' ')).parse_next(input)?;
    let _ = (',', space0).parse_next(input)?;
    let upper_bound =
        terminated(take_till(1.., |c| c == ']' || c == ' '), ']').parse_next(input)?;

    Ok(ModuleRetract::Range(
        lower_bound.to_string(),
        upper_bound.to_string(),
    ))
}

fn version_single(input: &mut LocatingSlice<&str>) -> Result<ModuleRetract> {
    let version = terminated(take_till(1.., WHITESPACES), multispace1).parse_next(input)?;

    Ok(ModuleRetract::Single(version.to_string()))
}

fn retract_multi(input: &mut LocatingSlice<&str>) -> Result<Vec<ModuleRetract>> {
    multi(input, retract_single)
}

fn ignore<'a>(input: &mut LocatingSlice<&'a str>) -> Result<Directive<'a>> {
    let res = preceded(
        ("ignore", space1),
        dispatch! {peek(any);
            '(' => ignore_multi,
            _ => ignore_single,
        },
    )
    .parse_next(input)?;
    let _ = take_while(0.., CRLF).parse_next(input)?;

    Ok(Directive::Ignore(res))
}

fn ignore_single(input: &mut LocatingSlice<&str>) -> Result<Vec<String>> {
    // terminate, if `)` is found
    peek(not(')')).parse_next(input)?;

    let path = take_till(1.., WHITESPACES).parse_next(input)?;

    // remove any comments added to the same line
    let _ = opt(comment).parse_next(input)?;

    Ok(vec![path.to_string()])
}

fn ignore_multi(input: &mut LocatingSlice<&str>) -> Result<Vec<String>> {
    multi(input, ignore_single)
}
