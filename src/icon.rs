use maud::{Markup, PreEscaped};

pub fn plus_icon() -> Markup {
    PreEscaped(include_str!("_asset/icon/plus.svg").to_string())
}
