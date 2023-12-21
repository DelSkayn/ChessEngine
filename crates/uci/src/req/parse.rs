use super::{GoRequest, OptionValue};
use crate::{r#move::UciMove, Request};
use common::board::Board;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, i32, i64, not_line_ending, space0, space1, u32, u64},
    combinator::{cut, eof, map, not, opt, recognize, value},
    error::ErrorKind,
    multi::{many1, separated_list1},
    sequence::{preceded, tuple},
    Err, IResult,
};

fn debug(i: &str) -> IResult<&str, Request> {
    let (i, _) = tag("debug")(i)?;
    let (i, _) = space1(i)?;
    let (i, x) = cut(alt((value(true, tag("on")), value(false, tag("off")))))(i)?;
    Ok((i, Request::Debug(x)))
}

fn option(i: &str) -> IResult<&str, Request> {
    let (i, _) = tag("setoption")(i)?;
    let (i, _) = space1(i)?;
    let (i, _) = cut(tag("name"))(i)?;
    let (i, _) = cut(space1)(i)?;

    let (i, name) = recognize(many1(tuple((
        not(tuple((tag("value"), space0))),
        tuple((alphanumeric1, space0)),
    ))))(i)?;

    let (i, value) = opt(preceded(
        tuple((tag("value"), space1)),
        cut(alt((
            map(i64, OptionValue::Spin),
            value(OptionValue::Check(true), tag("true")),
            value(OptionValue::Check(false), tag("false")),
            map(not_line_ending, |x: &str| OptionValue::String(x.to_owned())),
        ))),
    ))(i)?;

    Ok((
        i,
        Request::SetOption {
            name: name.to_owned(),
            value,
        },
    ))
}

fn position(i: &str) -> IResult<&str, Request> {
    let (i, _) = tag("position")(i)?;
    let (i, _) = space1(i)?;
    let (i, fen) = cut(alt((value(None, tag("startposition")), |i| {
        let (i, _) = tag("fen")(i)?;
        let (i, _) = space1(i)?;
        let Ok((board, i)) = Board::from_fen_partial(i) else {
            return Err(Err::Failure(nom::error::Error::new(i, ErrorKind::Fail)));
        };
        Ok((i, Some(Box::new(board))))
    })))(i)?;

    let (i, moves) = opt(|i| {
        let (i, _) = space1(i)?;
        let (i, _) = tag("moves")(i)?;
        let (i, _) = space1(i)?;
        cut(separated_list1(space1, UciMove::parse_partial))(i)
    })(i)?;

    Ok((
        i,
        Request::Position {
            fen,
            moves: moves.unwrap_or_default(),
        },
    ))
}

#[derive(Clone)]
enum GoOption {
    SearchMoves(Vec<UciMove>),
    Ponder,
    WTime(i64),
    BTime(i64),
    WInc(i32),
    BInc(i32),
    MovesToGo(u32),
    Depth(u32),
    Nodes(u64),
    Mate(u32),
    Movetime(u64),
    Infinite,
}

fn go_option(i: &str) -> IResult<&str, GoOption> {
    alt((
        |i| {
            let (i, _) = tag("searchmoves")(i)?;
            let (i, _) = space1(i)?;
            map(
                separated_list1(space1, UciMove::parse_partial),
                GoOption::SearchMoves,
            )(i)
        },
        value(GoOption::Ponder, tag("ponder")),
        map(
            preceded(tuple((tag("wtime"), space1)), i64),
            GoOption::WTime,
        ),
        map(
            preceded(tuple((tag("btime"), space1)), i64),
            GoOption::BTime,
        ),
        map(preceded(tuple((tag("winc"), space1)), i32), GoOption::WInc),
        map(preceded(tuple((tag("binc"), space1)), i32), GoOption::BInc),
        map(
            preceded(tuple((tag("movestogo"), space1)), u32),
            GoOption::MovesToGo,
        ),
        map(
            preceded(tuple((tag("depth"), space1)), u32),
            GoOption::Depth,
        ),
        map(
            preceded(tuple((tag("nodes"), space1)), u64),
            GoOption::Nodes,
        ),
        map(preceded(tuple((tag("mate"), space1)), u32), GoOption::Mate),
        map(
            preceded(tuple((tag("movetime"), space1)), u64),
            GoOption::Movetime,
        ),
        value(GoOption::Infinite, tag("infinite")),
    ))(i)
}

fn go(i: &str) -> IResult<&str, Request> {
    let (i, _) = tag("go")(i)?;
    let (i, options) = opt(|i| {
        let (i, _) = space1(i)?;
        separated_list1(space1, go_option)(i)
    })(i)?;

    let mut searchmoves = None;
    let mut ponder = false;
    let mut wtime = None;
    let mut btime = None;
    let mut winc = None;
    let mut binc = None;
    let mut moves_to_go = None;
    let mut depth = None;
    let mut nodes = None;
    let mut mate = None;
    let mut movetime = None;
    let mut infinite = false;

    for m in options.unwrap_or_default().into_iter() {
        match m {
            GoOption::SearchMoves(x) => searchmoves = Some(x),
            GoOption::Ponder => ponder = true,
            GoOption::WTime(x) => wtime = Some(x),
            GoOption::BTime(x) => btime = Some(x),
            GoOption::WInc(x) => winc = Some(x),
            GoOption::BInc(x) => binc = Some(x),
            GoOption::MovesToGo(x) => moves_to_go = Some(x),
            GoOption::Depth(x) => depth = Some(x),
            GoOption::Nodes(x) => nodes = Some(x),
            GoOption::Mate(x) => mate = Some(x),
            GoOption::Movetime(x) => movetime = Some(x),
            GoOption::Infinite => infinite = true,
        }
    }
    Ok((
        i,
        Request::Go(GoRequest {
            searchmoves,
            ponder,
            wtime,
            btime,
            winc,
            binc,
            moves_to_go,
            depth,
            nodes,
            mate,
            movetime,
            infinite,
        }),
    ))
}

pub fn parse(i: &str) -> IResult<&str, Request> {
    let (i, _) = space0(i)?;

    let (i, res) = alt((
        debug,
        value(Request::IsReady, tag("isready")),
        value(Request::UciNewGame, tag("ucinewgame")),
        value(Request::Uci, tag("uci")),
        option,
        position,
        go,
        value(Request::Stop, tag("stop")),
        value(Request::PonderHit, tag("ponderhit")),
        value(Request::Quit, tag("quit")),
    ))(i)?;

    let (i, _) = space0(i)?;
    let (i, _) = eof(i)?;
    Ok((i, res))
}
