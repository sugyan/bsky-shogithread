use shogi_core::Position;
use shogi_img::image::codecs::png::PngEncoder;

pub fn pos2img(pos: &Position) -> (Vec<u8>, (u32, u32)) {
    let mut buf = Vec::new();
    let img = shogi_img::pos2img(pos.inner());
    img.write_with_encoder(PngEncoder::new(&mut buf))
        .expect("failed to encode image");
    (buf, img.dimensions())
}
