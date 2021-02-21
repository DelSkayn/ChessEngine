use super::*;

use engine::Piece;
use ggez::{
    graphics::{self, Color, DrawMode, DrawParam, Image, Mesh, Rect},
    mint::Vector2,
    Context, GameResult,
};

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

pub fn draw_board(b: &Board, ctx: &mut Context, within: Rect, sprite: &Image) -> GameResult<()> {
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

    for piece in 0..12 {
        let param = piece_to_param(piece, [square_size, square_size], &sprite);
        for p in b[Piece::from_u8(piece)].iter() {
            let i = p.file();
            let j = 7 - p.rank();
            let x = offset_x + square_size * i as f32;
            let y = offset_y + square_size * j as f32;
            graphics::draw(ctx, sprite, param.dest([x, y]))?;
        }
    }

    Ok(())
}
