use chess_move_gen::{types::gen_type, InlineBuffer, MoveGenerator};
use common::board::Board;

fn main() {
    let move_gen = MoveGenerator::new();
    let mut board = Board::start_position();
    let mut buffer = InlineBuffer::new();

    let mut moves_made = Vec::new();

    move_gen.gen_moves::<gen_type::All>(&board, &mut buffer);

    let mut line_buffer = String::new();

    'main_loop: loop {
        std::io::stdin().read_line(&mut line_buffer).unwrap();

        match line_buffer.trim() {
            "u" => {
                if let Some(x) = moves_made.pop() {
                    board.unmake_move(x);
                    buffer.clear();
                    move_gen.gen_moves::<gen_type::All>(&board, &mut buffer);
                    println!("{board}");
                } else {
                    println!("no previous moves");
                }
            }
            "p" => {
                println!("{board}");
            }
            "d" => {
                println!("{board:?}");
            }
            "q" => return,
            "m" => {
                for m in buffer.iter() {
                    println!("move: {m}");
                }
            }
            "h" => {
                println!("hash: {}", board.hash);
            }
            "l" => {
                for m in buffer.iter() {
                    println!("move: {m}");
                }
            }
            "v" => {
                println!("board valid: {}", board.is_valid());
            }
            "f" => {
                println!("{}", board.to_fen());
            }
            x if x.starts_with('l') => {
                let str = &x[1..];
                if let Ok(new_board) = str.trim().parse() {
                    board = new_board;
                    moves_made.clear();
                    buffer.clear();
                    move_gen.gen_moves::<gen_type::All>(&board, &mut buffer);
                    println!("{board}");
                } else {
                    println!("failed to parse fen");
                }
            }
            _ => {
                for m in buffer.iter() {
                    if m.to_string() != line_buffer.trim() {
                        continue;
                    }
                    let m = board.make_move(m);
                    moves_made.push(m);
                    buffer.clear();
                    move_gen.gen_moves::<gen_type::All>(&board, &mut buffer);
                    line_buffer.clear();
                    println!("{board}");
                    continue 'main_loop;
                }
                println!("no such move found");
            }
        }

        line_buffer.clear()
    }
}
