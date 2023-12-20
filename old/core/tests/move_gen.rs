use chess_core::board::EndChain;
use rand::Rng;

#[test]
fn random_move_test() {
    use chess_core::{
        board::Board,
        gen::{gen_type, InlineBuffer, MoveGenerator, MoveList},
    };
    let move_gen = MoveGenerator::new();

    let mut moves = Vec::new();
    let mut boards = Vec::new();

    let mut board = Board::start_position(EndChain);
    let mut rng = rand::thread_rng();

    for _ in 0..1000 {
        let mut prev_board = board.clone();
        for _ in 0..100 {
            let mut moves_buffer = InlineBuffer::<128>::new();
            move_gen.gen_moves::<gen_type::All, _, EndChain>(&board, &mut moves_buffer);
            if moves_buffer.len() == 0 {
                break;
            }
            let pick = rng.gen_range(0..moves_buffer.len());
            //println!("Picked: {}",moves_buffer.get(pick));
            boards.push(board.clone());
            let m = moves_buffer.get(pick);
            let old_board = board.clone();
            let unmake_move = board.make_move(moves_buffer.get(pick));
            assert!(
                board.is_valid(),
                "move: {:?}\n{:#?}\nfen:{:?}\nfen before:{:?}\nmoves:{:#?}",
                m,
                board,
                old_board.to_fen(),
                prev_board.to_fen(),
                moves
            );
            prev_board = old_board;
            moves.push(unmake_move);
        }

        while let Some(x) = moves.pop() {
            board.unmake_move(x);
            assert!(board.is_valid(), "move: {:?},{:?}\n{:#?}", x, moves, board);
            assert!(board.is_equal(&boards.pop().unwrap()));
        }
        boards.pop();
    }
}
