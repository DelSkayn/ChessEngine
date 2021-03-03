use engine::{Board, Move, MoveGenerator, Piece, Square, BB};
use ggez::{
    event::MouseButton,
    graphics::{self, Color, DrawMode, DrawParam, Image, Mesh, Rect},
    input,
    mint::Vector2,
    Context, GameResult,
};

pub struct RenderBoard {
    pub board: Board,
    move_gen: MoveGenerator,
    selected: Option<Square>,
    dragging: Option<Piece>,
    holding: bool,
    moves: Vec<Move>,
    previous_move: Option<Move>,
    rect: Rect,
}

impl RenderBoard {
    pub fn new(board: Board) -> Self {
        let move_gen = MoveGenerator::new();
        let mut moves = Vec::new();
        move_gen.gen_moves(&board, &mut moves);
        RenderBoard {
            board,
            move_gen,
            moves,
            selected: None,
            dragging: None,
            holding: false,
            previous_move: None,
            rect: Rect::zero(),
        }
    }

    pub fn draw(&mut self, ctx: &mut Context, within: Rect, sprite: &Image) -> GameResult<()> {
        let max_size = within.w.min(within.h);
        let offset_x = (within.w - max_size).max(0.0) / 2.0;
        let offset_y = (within.h - max_size).max(0.0) / 2.0;
        let square_size = max_size / 8.0;

        // Draw the board
        for i in 0..8 {
            for j in 0..8 {
                let x = offset_x + square_size * i as f32;
                let y = offset_y + square_size * j as f32;
                let color = if (i + j) % 2 == 0 {
                    color_white()
                } else {
                    color_black()
                };

                let rect = Rect {
                    x,
                    y,
                    w: square_size,
                    h: square_size,
                };

                let rect = Mesh::new_rectangle(ctx, DrawMode::fill(), rect, color)?;
                graphics::draw(ctx, &rect, DrawParam::new())?;
            }
        }

        // Draw previous move
        match self.previous_move {
            None => {}
            Some(Move::Simple {
                mut from, mut to, ..
            }) => {
                if self.board.white_turn() {
                    from = from.flip();
                    to = to.flip();
                }

                let x = offset_x + square_size * from.file() as f32;
                let y = offset_y + square_size * (7 - from.rank()) as f32;
                let color = Color::from_rgb_u32(0x98971a);

                let rect = Rect {
                    x,
                    y,
                    w: square_size,
                    h: square_size,
                };

                let rect = Mesh::new_rectangle(ctx, DrawMode::fill(), rect, color)?;
                graphics::draw(ctx, &rect, DrawParam::new())?;

                let x = offset_x + square_size * to.file() as f32;
                let y = offset_y + square_size * (7 - to.rank()) as f32;

                let rect = Rect {
                    x,
                    y,
                    w: square_size,
                    h: square_size,
                };

                let rect = Mesh::new_rectangle(ctx, DrawMode::fill(), rect, color)?;
                graphics::draw(ctx, &rect, DrawParam::new())?;
            }
            _ => {}
        }

        if let Some(s) = self.selected {
            let x = offset_x + square_size * s.file() as f32;
            let y = offset_y + square_size * (7 - s.rank()) as f32;
            let color = Color::from_rgb_u32(0xd65d0e);

            let rect = Rect {
                x,
                y,
                w: square_size,
                h: square_size,
            };

            let rect = Mesh::new_rectangle(ctx, DrawMode::fill(), rect, color)?;
            graphics::draw(ctx, &rect, DrawParam::new())?;
        }

        // Draw all pieces except one that is dragged
        let exclude = if self.dragging.is_some() {
            BB::square(self.selected.unwrap())
        } else {
            BB::empty()
        };

        for piece in 0..12 {
            let param = piece_to_param(piece, [square_size, square_size], &sprite);
            for p in (self.board[Piece::from_u8(piece)] & !exclude).iter() {
                let i = p.file();
                let j = 7 - p.rank();
                let x = offset_x + square_size * i as f32;
                let y = offset_y + square_size * j as f32;
                graphics::draw(ctx, sprite, param.dest([x, y]))?;
            }
        }

        // Draw the dragged piece
        if let Some(x) = self.dragging {
            let param = piece_to_param(x as u8, [square_size, square_size], &sprite);
            let pos = input::mouse::position(&ctx);
            let pos = [pos.x - square_size / 2.0, pos.y - square_size / 2.0];
            graphics::draw(ctx, sprite, param.dest(pos))?;
        }

        self.rect = Rect {
            x: offset_x,
            y: offset_y,
            w: max_size,
            h: max_size,
        };
        Ok(())
    }

    pub fn mouse_motion_event(&mut self) {
        if self.holding {
            self.holding = false;
            if let Some(x) = self.selected {
                self.dragging = self.board.on(x);
            }
        }
    }

    pub fn mouse_button_down_event(&mut self, btn: MouseButton, x: f32, y: f32) {
        match btn {
            MouseButton::Left => {}
            MouseButton::Right => {
                self.selected = None;
                return;
            }
            _ => return,
        };
        if !self.rect.contains([x, y]) {
            return;
        }
        self.holding = true;
        let x = ((x - self.rect.x) / (self.rect.w / 8.0)).floor();
        let y = ((y - self.rect.y) / (self.rect.h / 8.0)).floor();

        let file = x as u8;
        let rank = 7 - y as u8;

        let square = Square::from_file_rank(file, rank);
        if self
            .board
            .on(square)
            .map(|x| x.white() == self.board.white_turn())
            .unwrap_or(false)
        {
            self.selected = Some(square);
        } else {
            self.selected = None;
        }
    }

    pub fn mouse_button_up_event(&mut self, btn: MouseButton, x: f32, y: f32) {
        match btn {
            MouseButton::Left => {}
            _ => return,
        };

        self.holding = false;
        if let Some(p) = self.dragging {
            if !self.rect.contains([x, y]) {
                return;
            }
            let x = ((x - self.rect.x) / (self.rect.w / 8.0)).floor();
            let y = ((y - self.rect.y) / (self.rect.h / 8.0)).floor();

            let file = x as u8;
            let rank = 7 - y as u8;

            let target = Square::from_file_rank(file, rank);
            println!(
                "move {:?} from {:?} to {:?}",
                p,
                self.selected.unwrap(),
                target
            );
            if self.play_move(self.selected.unwrap(), target) {
                self.selected = None;
            }
        }
        self.dragging = None;
    }

    pub fn play_move(&mut self, mut selected: Square, mut target: Square) -> bool {
        if !self.board.white_turn() {
            selected = selected.flip();
            target = target.flip();
        }
        let mut mov = None;
        for m in self.moves.iter() {
            dbg!(m);
            match *m {
                Move::Simple { from, to, .. } => {
                    if from == selected && to == target {
                        mov = Some(m);
                        break;
                    }
                }
                _ => return false,
            }
        }

        if let Some(x) = mov {
            if self.board.white_turn() {
                self.board = self.board.make_move(*x);
            } else {
                self.board = self.board.flip().make_move(*x).flip();
            }
            self.previous_move = Some(*x);
            dbg!(self.board);
            self.moves.clear();
            if !self.board.white_turn() {
                self.move_gen.gen_moves(&self.board.flip(), &mut self.moves);
            } else {
                self.move_gen.gen_moves(&self.board, &mut self.moves);
            }
            true
        } else {
            false
        }
    }
}

fn color_black() -> Color {
    Color::from_rgb(0x66, 0x5c, 0x54)
}

fn color_white() -> Color {
    Color::from_rgb(0xbd, 0xae, 0x93)
}

fn piece_to_param(piece: u8, scale: impl Into<Vector2<f32>>, sprite: &Image) -> DrawParam {
    let piece_size = sprite.dimensions();
    let piece_size = piece_size.w.max(piece_size.h) / 6.0;

    let scale = scale.into();
    let scale = Vector2 {
        x: scale.x / piece_size,
        y: scale.y / piece_size,
    };

    DrawParam {
        src: Rect {
            x: (piece % 6) as f32 / 6.0,
            y: (piece / 6) as f32 / 2.0,
            w: 1.0 / 6.0,
            h: 0.5,
        },
        ..DrawParam::new()
    }
    .scale(scale)
}
