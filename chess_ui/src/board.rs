use chess_core::{
    bb::BB,
    board::{Board, UnmakeMove},
    hash::Hasher,
    Move, Piece, Square,
};
use ggez::{
    graphics::{self, Color, DrawMode, DrawParam, Image, Mesh, Rect, Text},
    input,
    mint::Point2,
    mint::Vector2,
    Context, GameResult,
};

pub struct RenderBoard {
    hasher: Hasher,
    pub board: Board,
    selected: Option<Square>,
    possible_moves: Vec<Square>,
    dragging: Option<Square>,
    mov: Option<(Square, Square)>,
    rect: Rect,
    pub made_moves: Vec<UnmakeMove>,
}

impl RenderBoard {
    pub fn new(board: Board, hasher: Hasher) -> Self {
        RenderBoard {
            hasher,
            board,
            selected: None,
            dragging: None,
            possible_moves: Vec::new(),
            mov: None,
            rect: Rect::zero(),
            made_moves: Vec::new(),
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

                if self
                    .possible_moves
                    .contains(&Square::from_file_rank(i, 7 - j))
                {
                    let mut color = Color::from_rgb_u32(0x98971a);
                    color.a = 0.8;
                    let circle_size = square_size / 5.0;
                    let circle_offset = square_size / 2.0;
                    let circle = Mesh::new_circle(
                        ctx,
                        DrawMode::fill(),
                        [x + circle_offset, y + circle_offset],
                        circle_size,
                        0.1,
                        color,
                    )?;
                    graphics::draw(ctx, &circle, DrawParam::new())?;
                }
            }
        }

        let letter_offset_x = offset_x + square_size - (square_size / 6.0) - (square_size / 32.0);
        let letter_offset_y = offset_y + square_size - (square_size / 6.0) - (square_size / 32.0);
        for i in 0..8 {
            let mut text = Text::new(('A' as u8 + i) as char);
            text.set_font(Default::default(), (square_size / 6.0).into());
            let x = letter_offset_x + square_size * i as f32;
            let y = letter_offset_y + square_size * 7 as f32;
            let color = if (i) % 2 == 0 {
                color_white()
            } else {
                color_black()
            };
            graphics::draw(ctx, &text, ([x, y], color))?;
        }

        let letter_offset_x = offset_x + (square_size / 16.0);
        let letter_offset_y = offset_y + (square_size / 16.0);
        for i in 0..8 {
            let mut text = Text::new(('8' as u8 - i) as char);
            text.set_font(Default::default(), (square_size / 6.0).into());
            let x = letter_offset_x;
            let y = letter_offset_y + square_size * i as f32;
            let color = if (i) % 2 == 1 {
                color_white()
            } else {
                color_black()
            };
            graphics::draw(ctx, &text, ([x, y], color))?;
        }

        // Draw previous move
        if let Some((from, to)) = self.mov {
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
        let exclude = if let Some(x) = self.dragging {
            BB::square(x)
        } else {
            BB::empty()
        };

        for piece in 0..12 {
            let param = piece_to_param(piece, [square_size, square_size], &sprite);
            for p in (self.board.pieces[Piece::from_u8(piece)] & !exclude).iter() {
                let i = p.file();
                let j = 7 - p.rank();
                let x = offset_x + square_size * i as f32;
                let y = offset_y + square_size * j as f32;
                graphics::draw(ctx, sprite, param.dest([x, y]))?;
            }
        }

        // Draw the dragged piece
        if let Some(x) = self.dragging {
            let param = piece_to_param(
                self.on(x).unwrap() as u8,
                [square_size, square_size],
                &sprite,
            );
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

    pub fn set_possible(&mut self, moves: Vec<Square>) {
        self.possible_moves = moves;
    }

    pub fn make_move(&mut self, mov: Move) {
        let m = self.board.make_move(mov);
        self.made_moves.push(m);
        self.highlight(mov.from(), mov.to());
        self.possible_moves.clear();
        self.clear_drag();
    }

    /// Returns the square on the board for a specific mouse position
    pub fn square(&mut self, pos: impl Into<Point2<f32>>) -> Option<Square> {
        let pos = pos.into();
        if !self.rect.contains(pos) {
            return None;
        }

        let file = ((pos.x - self.rect.x) / (self.rect.w / 8.0)) as u8;
        let rank = ((pos.y - self.rect.y) / (self.rect.h / 8.0)) as u8;
        let rank = 7 - rank;

        let s = Square::from_file_rank(file, rank);
        println!("{}", s);
        Some(s)
    }

    pub fn undo_move(&mut self) {
        if self.made_moves.len() < 2 {
            return;
        }
        if let Some(x) = self.made_moves.pop() {
            self.board.unmake_move(x);
        }
        /*
        if let Some(x) = self.made_moves.pop() {
            self.board.unmake_move(x, &self.hasher);
        }
        */
        self.clear_highlight();
        self.clear_select();
        self.possible_moves.clear();
    }

    pub fn on(&self, square: Square) -> Option<Piece> {
        self.board.on(square)
    }

    /// Select a specific square on the board.
    pub fn select(&mut self, square: Square) {
        self.selected = Some(square);
    }

    /// Clear the selected square on the board
    pub fn clear_select(&mut self) {
        self.possible_moves.clear();
        self.selected = None;
    }

    /// highlight a move on the board
    pub fn highlight(&mut self, from: Square, to: Square) {
        self.mov = Some((from, to));
    }

    /// Clear the highlighted move.
    pub fn clear_highlight(&mut self) {
        self.mov = None;
    }

    /// Set a piece to be dragged
    pub fn drag(&mut self, square: Square) -> Option<Piece> {
        if let Some(piece) = self.on(square) {
            self.dragging = Some(square);
            return Some(piece);
        }
        return None;
    }

    pub fn clear_drag(&mut self) {
        self.dragging = None;
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
