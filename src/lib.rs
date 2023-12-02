/// Returns `ggez::graphics::Color` value, as const
macro_rules! color {
    (? $rng:expr $(,)?) => {
        color!(
            $rng.gen_range(0..=255),
            $rng.gen_range(0..=255),
            $rng.gen_range(0..=255),
        )
    };

    ($name:ident $(,)?) => {
        ::ggez::graphics::Color::$name
    };
    ($hex:literal $(,)?) => {
        color!(($hex >> 16) & 0xFF, ($hex >> 8) & 0xFF, $hex & 0xFF,)
    };
    ($r:expr, $g:expr, $b:expr $(,)?) => {
        ::ggez::graphics::Color::new(
            $r as u8 as f32 / 255.0,
            $g as u8 as f32 / 255.0,
            $b as u8 as f32 / 255.0,
            255.0,
        )
    };
    ($r:expr, $g:expr, $b:expr, $a:expr $(,)?) => {
        ::ggez::graphics::Color::new(
            $r as u8 as f32 / 255.0,
            $g as u8 as f32 / 255.0,
            $b as u8 as f32 / 255.0,
            $a as u8 as f32 / 255.0,
        )
    };
}

mod app;

pub use app::App;
