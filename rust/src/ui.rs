use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::Point,
    mono_font::{
        ascii::{FONT_10X20, FONT_6X10},
        MonoTextStyle,
    },
    pixelcolor::BinaryColor,
    text::{Alignment, Text},
    Drawable,
};

pub const W: i32 = 128;
pub const H: i32 = 64;

pub fn draw_centered<D: DrawTarget<Color = BinaryColor>>(
    target: &mut D,
    text: &str,
    y: i32,
    style: MonoTextStyle<'static, BinaryColor>,
) -> Result<(), D::Error> {
    Text::with_alignment(text, Point::new(W / 2, y), style, Alignment::Center).draw(target)?;
    Ok(())
}

pub fn title_style() -> MonoTextStyle<'static, BinaryColor> {
    MonoTextStyle::new(&FONT_10X20, BinaryColor::On)
}

pub fn body_style() -> MonoTextStyle<'static, BinaryColor> {
    MonoTextStyle::new(&FONT_6X10, BinaryColor::On)
}
