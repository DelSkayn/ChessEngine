use common::{board::Board, Piece, Player};

pub const POSITION_TENSOR_LEN: usize = 13 * 8 * 8;

pub fn to_tensor(b: &Board, buffer: &mut [u8]) {
    for (idx, p) in Piece::player_pieces(b.state.player).enumerate() {
        for s in b.pieces[p].iter() {
            let s = if b.state.player == Player::Black {
                s.flip()
            } else {
                s
            };
            buffer[idx * 64 + s.rank() as usize * 8 + s.file() as usize] = 1;
        }
    }
    for (idx, p) in Piece::player_pieces(b.state.player.flip()).enumerate() {
        for s in b.pieces[p].iter() {
            let s = if b.state.player == Player::Black {
                s.flip()
            } else {
                s
            };
            buffer[(idx + 6) * 64 + s.rank() as usize * 8 + s.file() as usize] = 1;
        }
    }
}
