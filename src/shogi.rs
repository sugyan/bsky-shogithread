use crate::error::{Error, Result};
use shogi_core::{LegalityChecker, Move, Piece, Position};
use shogi_img::{image::codecs::png::PngEncoder, Generator, HighlightSquare};
use shogi_kifu_converter::{converter::ToKi2, parser::parse_ki2_str, JKF};
use shogi_legality_lite::LiteLegalityChecker;
use shogi_usi_parser::FromUsi;

pub struct MoveChecker {
    pos: Position,
    ki2: String,
}

impl MoveChecker {
    pub fn new(pos: Position) -> Result<Self> {
        let jkf = JKF::try_from(&pos)?;
        let ki2 = jkf.to_ki2_owned().trim().to_string();
        Ok(Self { pos, ki2 })
    }
    pub fn try_move(&mut self, text: &str) -> Result<&Position> {
        let s = text.split_whitespace().next().unwrap_or_default();
        // first, try to parse as usi format
        let mv = Move::from_usi(text)
            .map(|mut mv| {
                // https://docs.rs/shogi_usi_parser/latest/shogi_usi_parser/trait.FromUsi.html#impl-FromUsi-for-Move
                if let Move::Drop { piece, to: _ } = &mut mv {
                    *piece = Piece::new(piece.piece_kind(), self.pos.side_to_move());
                }
                mv
            })
            // next, try to parse as ki2 format
            .or_else(|_| {
                let mut s = s.to_string();
                for (from, to) in [
                    ('1', "１"),
                    ('2', "２"),
                    ('3', "３"),
                    ('4', "４"),
                    ('5', "５"),
                    ('6', "６"),
                    ('7', "７"),
                    ('8', "８"),
                    ('9', "９"),
                ] {
                    s = s.replace(from, to);
                }
                let jkf = parse_ki2_str(&format!("{} {s}", self.ki2))?;
                Ok::<Move, Error>(Position::try_from(&jkf)?.last_move().unwrap())
            })?;
        LiteLegalityChecker
            .make_move(&mut self.pos, mv)
            .map_err(Error::ShogiCoreIllegaleMove)?;
        Ok(&self.pos)
    }
}

pub fn pos2img(pos: &Position) -> (Vec<u8>, (u32, u32)) {
    let mut buf = Vec::new();
    let img = Generator::new(
        Default::default(),
        Default::default(),
        HighlightSquare::LastMoveTo,
    )
    .generate(pos.inner());
    img.write_with_encoder(PngEncoder::new(&mut buf))
        .expect("failed to encode image");
    (buf, img.dimensions())
}

pub fn last_move_ki2(pos: &Position) -> Result<Option<String>> {
    let jkf = JKF::try_from(pos)?;
    Ok(jkf
        .to_ki2_owned()
        .split_whitespace()
        .last()
        .map(String::from))
}
