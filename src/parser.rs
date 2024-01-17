use crate::combinator::not_whitespace;
use crate::{Module, ModuleDependency, ModuleReplacement, ModuleRetract, Replacement};
use winnow::ascii::{multispace0, multispace1, space0, space1};
use winnow::combinator::{fail, not, opt, peek, preceded, repeat, terminated};
use winnow::stream::AsChar;
use winnow::token::{any, take_till, take_while};
use winnow::{dispatch, PResult, Parser};

const WHITESPACES: [char; 3] = [' ', '\t', '\n'];

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Directive<'a> {
    Comment(&'a str),
    Module(&'a str),
    Go(&'a str),
    Toolchain(&'a str),
    Require(Vec<ModuleDependency>),
    Exclude(Vec<ModuleDependency>),
    Replace(Vec<ModuleReplacement>),
    Retract(Vec<ModuleRetract>),
}

pub(crate) fn gomod<'a>(input: &mut &'a str) -> PResult<Vec<Directive<'a>>> {
    repeat(0.., directive).parse_next(input)
}

fn directive<'a>(input: &mut &'a str) -> PResult<Directive<'a>> {
    let _ = take_while(0.., AsChar::is_newline).parse_next(input)?;
    dispatch!(peek(not_whitespace);
        "//" => comment,
        "module" => module,
        "go" => go,
        "toolchain" => toolchain,
        "require" => require,
        "exclude" => exclude,
        "replace" => replace,
        "retract" => retract,
        _ => fail,
    )
    .parse_next(input)
}

fn comment<'a>(input: &mut &'a str) -> PResult<Directive<'a>> {
    let res = preceded(
        (opt(space0), "//", opt(space0)),
        take_till(0.., AsChar::is_newline),
    )
    .parse_next(input)?;
    let _ = take_while(1.., AsChar::is_newline).parse_next(input)?;

    Ok(Directive::Comment(res))
}

fn module<'a>(input: &mut &'a str) -> PResult<Directive<'a>> {
    let res = preceded(("module", space1), take_till(1.., AsChar::is_newline)).parse_next(input)?;
    let _ = take_while(1.., AsChar::is_newline).parse_next(input)?;

    Ok(Directive::Module(res))
}

fn go<'a>(input: &mut &'a str) -> PResult<Directive<'a>> {
    let res = preceded(("go", space1), take_till(1.., AsChar::is_newline)).parse_next(input)?;
    let _ = take_while(1.., AsChar::is_newline).parse_next(input)?;

    Ok(Directive::Go(res))
}

fn toolchain<'a>(input: &mut &'a str) -> PResult<Directive<'a>> {
    let res =
        preceded(("toolchain", space1), take_till(1.., AsChar::is_newline)).parse_next(input)?;
    let _ = take_while(1.., AsChar::is_newline).parse_next(input)?;

    Ok(Directive::Toolchain(res))
}

fn require<'a>(input: &mut &'a str) -> PResult<Directive<'a>> {
    let res = preceded(
        ("require", space1),
        dispatch! {peek(any);
            '(' => require_multi,
            _ => require_single,
        },
    )
    .parse_next(input)?;
    let _ = take_while(0.., AsChar::is_newline).parse_next(input)?;

    Ok(Directive::Require(res))
}

fn require_single(input: &mut &str) -> PResult<Vec<ModuleDependency>> {
    // terminate, if `)` is found
    peek(not(')')).parse_next(input)?;

    let (module_path, _, version) = (
        take_till(1.., AsChar::is_space),
        space1,
        take_till(1.., WHITESPACES),
    )
        .parse_next(input)?;

    let indirect = opt(comment)
        .parse_next(input)?
        .map_or(false, |d| d == Directive::Comment("indirect"));

    Ok(vec![ModuleDependency {
        module: Module {
            module_path: module_path.to_string(),
            version: version.to_string(),
        },
        indirect,
    }])
}

fn require_multi(input: &mut &str) -> PResult<Vec<ModuleDependency>> {
    let _ = ("(", multispace1).parse_next(input)?;
    let res: Vec<Vec<ModuleDependency>> =
        repeat(1.., terminated(require_single, multispace0)).parse_next(input)?;
    let _ = (")", multispace1).parse_next(input)?;

    Ok(res.into_iter().flatten().collect::<Vec<ModuleDependency>>())
}

fn exclude<'a>(input: &mut &'a str) -> PResult<Directive<'a>> {
    let res = preceded(
        ("exclude", space1),
        dispatch! {peek(any);
            '(' => require_multi,
            _ => require_single,
        },
    )
    .parse_next(input)?;
    let _ = take_while(0.., AsChar::is_newline).parse_next(input)?;

    Ok(Directive::Exclude(res))
}

fn replace<'a>(input: &mut &'a str) -> PResult<Directive<'a>> {
    let res = preceded(
        ("replace", space1),
        dispatch! {peek(any);
            '(' => replace_multi,
            _ => replace_single,
        },
    )
    .parse_next(input)?;
    let _ = take_while(0.., AsChar::is_newline).parse_next(input)?;

    Ok(Directive::Replace(res))
}

fn replace_single(input: &mut &str) -> PResult<Vec<ModuleReplacement>> {
    // terminate, if `)` is found
    peek(not(')')).parse_next(input)?;

    let (src_path, src_version) = (
        terminated(take_till(1.., AsChar::is_space), space1),
        opt(terminated(
            preceded(peek(not("=>")), take_till(1.., AsChar::is_space)),
            space1,
        )),
    )
        .parse_next(input)?;
    let _ = ("=>", space1).parse_next(input)?;
    let (dest_path, dest_version) = (
        terminated(take_till(1.., WHITESPACES), space0),
        opt(terminated(take_till(1.., WHITESPACES), multispace1)),
    )
        .parse_next(input)?;

    let replacement = dest_version.map_or_else(
        || Replacement::FilePath(dest_path.to_string()),
        |version| {
            Replacement::Module(Module {
                module_path: dest_path.to_string(),
                version: version.to_string(),
            })
        },
    );

    Ok(vec![ModuleReplacement {
        module_path: src_path.to_string(),
        version: src_version.map(ToString::to_string),
        replacement,
    }])
}

fn replace_multi(input: &mut &str) -> PResult<Vec<ModuleReplacement>> {
    let _ = ("(", multispace1).parse_next(input)?;
    let res: Vec<Vec<ModuleReplacement>> =
        repeat(1.., terminated(replace_single, multispace0)).parse_next(input)?;
    let _ = (")", multispace1).parse_next(input)?;

    Ok(res
        .into_iter()
        .flatten()
        .collect::<Vec<ModuleReplacement>>())
}

fn retract<'a>(input: &mut &'a str) -> PResult<Directive<'a>> {
    let res = preceded(
        ("retract", space1),
        dispatch! {peek(any);
            '(' => retract_multi,
            _ => retract_single,
        },
    )
    .parse_next(input)?;
    let _ = take_while(0.., AsChar::is_newline).parse_next(input)?;

    Ok(Directive::Retract(res))
}

fn retract_single(input: &mut &str) -> PResult<Vec<ModuleRetract>> {
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

fn version_range(input: &mut &str) -> PResult<ModuleRetract> {
    let lower_bound = preceded('[', take_till(1.., |c| c == ',' || c == ' ')).parse_next(input)?;
    let _ = (',', space0).parse_next(input)?;
    let upper_bound =
        terminated(take_till(1.., |c| c == ']' || c == ' '), ']').parse_next(input)?;

    Ok(ModuleRetract::Range(
        lower_bound.to_string(),
        upper_bound.to_string(),
    ))
}

fn version_single(input: &mut &str) -> PResult<ModuleRetract> {
    let version = terminated(take_till(1.., WHITESPACES), multispace1).parse_next(input)?;

    Ok(ModuleRetract::Single(version.to_string()))
}

fn retract_multi(input: &mut &str) -> PResult<Vec<ModuleRetract>> {
    let _ = ("(", multispace1).parse_next(input)?;
    let res: Vec<Vec<ModuleRetract>> =
        repeat(1.., terminated(retract_single, multispace0)).parse_next(input)?;
    let _ = (")", multispace1).parse_next(input)?;

    Ok(res.into_iter().flatten().collect::<Vec<ModuleRetract>>())
}
