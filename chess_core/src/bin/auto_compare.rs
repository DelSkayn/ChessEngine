use chess_core::{
    board::{Board, EndChain},
    gen::{gen_type, InlineBuffer, MoveGenerator, MoveList},
    Move,
};
use std::{
    env,
    io::{BufRead, BufReader, Result, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

pub struct StockFish {
    _child: Child,
    sin: ChildStdin,
    sout: BufReader<ChildStdout>,
}

impl StockFish {
    pub fn new() -> Result<Self> {
        let mut child = Command::new("stockfish")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        Ok(StockFish {
            sin: child.stdin.take().unwrap(),
            sout: BufReader::new(child.stdout.take().unwrap()),
            _child: child,
        })
    }

    pub fn set_moves(&mut self, b: &Board, moves: &[Move]) -> Result<()> {
        write!(self.sin, "position fen {} moves ", b.to_fen())?;
        for m in moves.iter() {
            write!(self.sin, " {}", m)?
        }
        writeln!(self.sin)?;
        Ok(())
    }

    pub fn draw(&mut self) -> Result<()> {
        writeln!(self.sin, "d")?;
        let mut buffer = String::new();
        loop {
            buffer.clear();
            self.sout.read_line(&mut buffer)?;
            print!("{}", buffer);
            if buffer.contains("Checkers") {
                break;
            }
        }
        Ok(())
    }

    pub fn perft(&mut self, depth: usize) -> Result<Vec<(Move, usize)>> {
        writeln!(self.sin, "go perft {}", depth)?;

        let mut res = Vec::new();
        let mut buffer = String::new();
        loop {
            buffer.clear();
            self.sout.read_line(&mut buffer)?;
            if let Some(x) = buffer.find(":") {
                let (first, rest) = buffer.split_at(x);
                let rest = &rest[1..];
                if first == "Nodes searched" {
                    break;
                }
                res.push((
                    Move::from_name(first).unwrap(),
                    rest.trim().parse::<usize>().unwrap(),
                ));
            }
        }

        Ok(res)
    }
}

fn compare(my: Vec<(Move, usize)>, mut their: Vec<(Move, usize)>) -> Option<(Move, usize, usize)> {
    for (mov, count) in my {
        let stock_mov_idx =
            their
                .iter()
                .enumerate()
                .find_map(|(idx, x)| if x.0 == mov { Some(idx) } else { None });

        if let Some(idx) = stock_mov_idx {
            let (smov, scount) = their.swap_remove(idx);
            println!("{}:m={},s={}", smov, count, scount);
            if scount != count {
                return Some((mov, count, scount));
            }
        } else {
            println!("move {}:{} with missing from stockfish", mov, count);
        }
    }
    for (m, c) in their {
        println!("move {}:{} missing.", m, c);
    }
    None
}

fn main() -> Result<()> {
    let mut args = env::args();
    args.next();
    let depth: usize = args.next().unwrap().parse().unwrap();
    let mut board = if let Some(x) = args.next() {
        Board::from_fen(&x, EndChain).unwrap()
    } else {
        Board::start_position(EndChain)
    };

    let mut stockfish = StockFish::new()?;
    stockfish.set_moves(&board, &[])?;

    let move_gen = MoveGenerator::new();

    let mut count = 0;
    println!("running perft depth {}", depth);
    let my_perft = perft(&move_gen, &mut board, depth, &mut count);
    println!("running stockfish perft");
    let stock_perft = stockfish.perft(depth)?;

    let invalid_move = compare(my_perft, stock_perft);
    if let Some(x) = invalid_move {
        println!("Wrong move: {}, got {} should be {}", x.0, x.1, x.2);
    } else {
        println!("No discrepencies found");
        return Ok(());
    }

    let m = invalid_move.unwrap().0;
    let mut moves = vec![m];

    for i in (1..depth).rev() {
        stockfish.set_moves(&board, &moves)?;
        stockfish.draw()?;
        let mut board = board.clone();
        for m in moves.iter().copied() {
            board.make_move(m);
        }

        println!("running perft");
        let my_perft = perft(&move_gen, &mut board, i, &mut count);
        for (m, c) in my_perft.iter() {
            println!("{}\t{}", m, c);
        }
        println!("running stockfish perft");
        let stock_perft = stockfish.perft(i)?;
        for (m, c) in my_perft.iter() {
            println!("{}\t{}", m, c);
        }

        let invalid_move = compare(my_perft, stock_perft);
        if let Some(invalid_move) = invalid_move {
            moves.push(invalid_move.0);
        } else {
            break;
        }
    }

    println!("Invalid path:");
    for m in moves.iter() {
        println!("{}", m);
    }

    Ok(())
}

fn perft(
    gen: &MoveGenerator,
    b: &mut Board,
    depth: usize,
    count: &mut usize,
) -> Vec<(Move, usize)> {
    let mut buffer = InlineBuffer::<128>::new();
    gen.gen_moves::<gen_type::All, _, EndChain>(b, &mut buffer);
    let mut res = Vec::new();
    for i in 0..buffer.len() {
        let m = buffer.get(i);
        let last = *count;
        let m = b.make_move(m);
        perft_rec(gen, b, depth - 1, count);
        b.unmake_move(m);
        //println!("{}:{}", m.mov, cnt);
        res.push((m.mov, *count - last));
    }
    res
}

fn perft_rec(gen: &MoveGenerator, b: &mut Board, depth: usize, count: &mut usize) {
    if depth == 0 {
        *count += 1;
        return;
    }
    let mut buffer = InlineBuffer::<128>::new();
    gen.gen_moves::<gen_type::All, _, EndChain>(b, &mut buffer);
    for i in 0..buffer.len() {
        let m = buffer.get(i);
        let m = b.make_move(m);
        perft_rec(gen, b, depth - 1, count);
        b.unmake_move(m);
    }
}
