use super::*;

use ggez::{
    graphics::{self, Color, DrawMode, DrawParam, Image, Mesh, Rect},
    mint::Vector2,
    Context, GameResult,
};

fn color_black() -> Color {
    Color::from_rgb(0x3c, 0x38, 0x36)
}

fn color_white() -> Color {
    Color::from_rgb(0xfb, 0xf1, 0xc7)
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

impl Board {
    pub fn draw(&self, ctx: &mut Context, within: Rect, sprite: &Image) -> GameResult<()> {
        let max_size = within.w.min(within.h);
        let offset_x = (within.w - max_size).max(0.0) / 2.0;
        let offset_y = (within.h - max_size).max(0.0) / 2.0;
        let square_size = max_size / 8.0;

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

        let mut render_piece = |piece, map: u64| {
            let param = piece_to_param(piece, [square_size, square_size], &sprite);
            for i in 0..8 {
                for j in 0..8 {
                    if map & (1 << j * 8 + i) != 0 {
                        let x = offset_x + square_size * i as f32;
                        let y = offset_y + square_size * j as f32;

                        graphics::draw(ctx, sprite, param.dest([x, y]))?;
                    }
                }
            }
            GameResult::Ok(())
        };

        render_piece(0, self.white_king)?;
        render_piece(1, self.white_queens)?;
        render_piece(2, self.white_bishops)?;
        render_piece(3, self.white_knights)?;
        render_piece(4, self.white_rooks)?;
        render_piece(5, self.white_pawns)?;

        render_piece(6, self.black_king)?;
        render_piece(7, self.black_queens)?;
        render_piece(8, self.black_bishops)?;
        render_piece(9, self.black_knights)?;
        render_piece(10, self.black_rooks)?;
        render_piece(11, self.black_pawns)?;

        Ok(())
    }
}
