use engine::{hash::Hasher, Board, MoveGenerator};
use rand::Rng;

#[test]
fn random_move_test() {
    let hasher = Hasher::new();
    let move_gen = MoveGenerator::new();

    let mut moves = Vec::new();
    let mut boards = Vec::new();
    let mut moves_buffer = Vec::new();

    let mut board = Board::start_position();
    let mut rng = rand::thread_rng();

    for _ in 0..10000 {
        for _ in 0..100 {
            moves_buffer.clear();
            move_gen.gen_moves(&board, &mut moves_buffer);
            if moves_buffer.len() == 0 {
                break;
            }
            let pick = rng.gen_range(0..moves_buffer.len());
            boards.push(board.clone());
            let unmake_move = board.make_move(moves_buffer[pick], &hasher);
            board.assert_valid();
            moves.push(unmake_move);
        }

        while let Some(x) = moves.pop() {
            board.unmake_move(x);
            assert_eq!(board, boards.pop().unwrap());
        }
        boards.pop();
    }
}
