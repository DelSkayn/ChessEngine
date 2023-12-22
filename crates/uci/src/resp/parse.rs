use super::{
    OptionKind, Response, ResponseBound, ResponseId, ResponseInfo, ResponseOption, ResponseScore,
};
use crate::UciMove;
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{i64, not_line_ending, space0, space1, u16, u32, u64},
    combinator::{cut, eof, map, opt, value},
    multi::separated_list1,
    sequence::{preceded, terminated, tuple},
    IResult,
};

enum ComboKindOption {
    Default(String),
    Var(String),
}

fn combo_kind_option(i: &str) -> IResult<&str, ComboKindOption> {
    alt((
        map(
            preceded(
                tuple((tag("default"), space1)),
                map(take_while1(|x: char| !x.is_whitespace()), |x: &str| {
                    x.to_owned()
                }),
            ),
            ComboKindOption::Default,
        ),
        map(
            preceded(
                tuple((tag("var"), space1)),
                map(take_while1(|x: char| !x.is_whitespace()), |x: &str| {
                    x.to_owned()
                }),
            ),
            ComboKindOption::Var,
        ),
    ))(i)
}

enum SpinKindOption {
    Default(i64),
    Min(i64),
    Max(i64),
}

fn spin_kind_option(i: &str) -> IResult<&str, SpinKindOption> {
    alt((
        map(
            preceded(tuple((tag("default"), space1)), i64),
            SpinKindOption::Default,
        ),
        map(
            preceded(tuple((tag("min"), space1)), i64),
            SpinKindOption::Min,
        ),
        map(
            preceded(tuple((tag("max"), space1)), i64),
            SpinKindOption::Max,
        ),
    ))(i)
}

fn option_kind(i: &str) -> IResult<&str, OptionKind> {
    alt((
        |i| {
            let (i, _) = tag("spin")(i)?;
            let (i, options) = opt(preceded(space1, separated_list1(space1, spin_kind_option)))(i)?;
            let mut default = None;
            let mut min = None;
            let mut max = None;
            for o in options.unwrap_or_default() {
                match o {
                    SpinKindOption::Default(x) => default = Some(x),
                    SpinKindOption::Min(x) => min = Some(x),
                    SpinKindOption::Max(x) => max = Some(x),
                }
            }
            Ok((i, OptionKind::Spin { default, min, max }))
        },
        |i| {
            let (i, _) = tag("check")(i)?;
            let (i, default) = opt(preceded(
                tuple((space1, tag("default"), space1)),
                alt((value(true, tag("true")), value(false, tag("false")))),
            ))(i)?;
            Ok((i, OptionKind::Check { default }))
        },
        |i| {
            let (i, _) = tag("string")(i)?;
            let (i, default) = opt(preceded(
                tuple((space1, tag("default"), space1)),
                map(not_line_ending, |x: &str| x.to_owned()),
            ))(i)?;
            Ok((i, OptionKind::String { default }))
        },
        |i| {
            let (i, _) = tag("button")(i)?;
            Ok((i, OptionKind::Button))
        },
        |i| {
            let (i, _) = tag("combo")(i)?;
            let (i, options) =
                opt(preceded(space1, separated_list1(space1, combo_kind_option)))(i)?;
            let mut default = None;
            let mut values = Vec::new();
            for o in options.unwrap_or_default() {
                match o {
                    ComboKindOption::Default(x) => default = Some(x),
                    ComboKindOption::Var(x) => {
                        values.push(x);
                    }
                }
            }
            Ok((
                i,
                OptionKind::Combo {
                    default,
                    options: values,
                },
            ))
        },
    ))(i)
}

fn option(i: &str) -> IResult<&str, ResponseOption> {
    let (i, _) = tag("option")(i)?;
    let (i, _) = space1(i)?;
    let (i, _) = tag("name")(i)?;
    let (i, _) = space1(i)?;

    let (i, name) = take_while1(|x: char| !x.is_whitespace())(i)?;

    let (i, _) = space1(i)?;
    let (i, _) = tag("type")(i)?;
    let (i, _) = space1(i)?;
    let (i, kind) = option_kind(i)?;

    Ok((
        i,
        ResponseOption {
            name: name.to_owned(),
            kind,
        },
    ))
}

#[derive(Clone)]
enum ScoreOptions {
    Mate(u32),
    Cp(i64),
    Bound(ResponseBound),
}

fn score_options(i: &str) -> IResult<&str, ScoreOptions> {
    alt((
        map(
            preceded(tuple((tag("mate"), space1)), u32),
            ScoreOptions::Mate,
        ),
        map(preceded(tuple((tag("cp"), space1)), i64), ScoreOptions::Cp),
        value(
            ScoreOptions::Bound(ResponseBound::Lowerbound),
            tag("lowerbound"),
        ),
        value(
            ScoreOptions::Bound(ResponseBound::Upperbound),
            tag("upperbound"),
        ),
    ))(i)
}

fn score(i: &str) -> IResult<&str, ResponseScore> {
    let (i, _) = tag("score")(i)?;
    let (i, _) = space1(i)?;

    let (i, options) = separated_list1(space1, score_options)(i)?;
    let mut res = ResponseScore {
        mate: None,
        score: None,
        bound: ResponseBound::Exact,
    };
    for o in options {
        match o {
            ScoreOptions::Mate(x) => res.mate = Some(x),
            ScoreOptions::Cp(x) => res.score = Some(x),
            ScoreOptions::Bound(x) => res.bound = x,
        }
    }
    Ok((i, res))
}

fn resp_info(i: &str) -> IResult<&str, ResponseInfo> {
    alt((
        map(
            preceded(tuple((tag("depth"), space1)), u32),
            ResponseInfo::Depth,
        ),
        map(
            preceded(tuple((tag("seldepth"), space1)), u32),
            ResponseInfo::SelectiveDepth,
        ),
        map(
            preceded(tuple((tag("time"), space1)), u64),
            ResponseInfo::Time,
        ),
        map(
            preceded(tuple((tag("nodes"), space1)), u64),
            ResponseInfo::Nodes,
        ),
        map(
            preceded(tuple((tag("nps"), space1)), u64),
            ResponseInfo::Nps,
        ),
        |i| {
            let (i, _) = tag("pv")(i)?;
            let (i, _) = space1(i)?;
            let (i, m) = separated_list1(space1, UciMove::parse_partial)(i)?;
            Ok((i, ResponseInfo::Pv(m)))
        },
        map(
            preceded(tuple((tag("multipv"), space1)), u32),
            ResponseInfo::MultiPv,
        ),
        map(score, ResponseInfo::Score),
        map(
            preceded(tuple((tag("currmove"), space1)), UciMove::parse_partial),
            ResponseInfo::CurrMove,
        ),
        map(
            preceded(tuple((tag("currmovenumber"), space1)), u32),
            ResponseInfo::CurrMoveNumber,
        ),
        map(
            preceded(tuple((tag("hashfull"), space1)), u16),
            ResponseInfo::Hashfull,
        ),
        map(
            preceded(tuple((tag("tbhits"), space1)), u64),
            ResponseInfo::TbHits,
        ),
        map(
            preceded(tuple((tag("sbhits"), space1)), u64),
            ResponseInfo::SbHits,
        ),
        map(
            preceded(tuple((tag("cpuload"), space1)), u16),
            ResponseInfo::CpuLoad,
        ),
        map(
            preceded(tuple((tag("string"), space1)), not_line_ending),
            |x: &str| ResponseInfo::String(x.to_owned()),
        ),
        |i| {
            let (i, _) = tag("refutation")(i)?;
            let (i, _) = space1(i)?;
            let (i, m) = separated_list1(space1, UciMove::parse_partial)(i)?;
            Ok((i, ResponseInfo::Refutation(m)))
        },
        |i| {
            let (i, _) = tag("currline")(i)?;
            let (i, _) = space1(i)?;
            let (i, cpu) = opt(terminated(u32, space1))(i)?;
            let (i, m) = separated_list1(space1, UciMove::parse_partial)(i)?;
            Ok((i, ResponseInfo::CurrLine { cpu, moves: m }))
        },
    ))(i)
}

fn info(i: &str) -> IResult<&str, Response> {
    let (i, _) = tag("info")(i)?;
    let (i, _) = space1(i)?;
    let (i, info) = cut(separated_list1(space1, resp_info))(i)?;
    Ok((i, Response::Info(info)))
}

fn bestmove(i: &str) -> IResult<&str, Response> {
    let (i, _) = tag("bestmove")(i)?;
    let (i, _) = space1(i)?;
    let (i, m) = cut(UciMove::parse_partial)(i)?;

    let (i, ponder) = opt(preceded(
        tuple((space1, tag("ponder"), space1)),
        cut(UciMove::parse_partial),
    ))(i)?;

    Ok((i, Response::BestMove { r#move: m, ponder }))
}

fn id(i: &str) -> IResult<&str, ResponseId> {
    let (i, _) = tag("id")(i)?;
    let (i, _) = space1(i)?;
    alt((
        map(
            preceded(tuple((tag("author"), space1)), not_line_ending),
            |x: &str| ResponseId::Author(x.to_owned()),
        ),
        map(
            preceded(tuple((tag("name"), space1)), not_line_ending),
            |x: &str| ResponseId::Name(x.to_owned()),
        ),
    ))(i)
}

pub fn parse(i: &str) -> IResult<&str, Response> {
    let (i, _) = space0(i)?;

    let (i, res) = alt((
        value(Response::UciOk, tag("uciok")),
        value(Response::ReadyOk, tag("readyok")),
        map(id, Response::Id),
        bestmove,
        info,
        map(option, Response::Option),
    ))(i)?;

    let (i, _) = space0(i)?;
    let (i, _) = eof(i)?;

    Ok((i, res))
}
