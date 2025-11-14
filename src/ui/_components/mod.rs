use gpui::*;
use gpui_component::{Icon, IconName};

pub mod nav;
pub use nav::*;

pub fn loading_icon(f: impl FnOnce(Icon) -> Icon) -> impl IntoElement {
    f(Icon::new(IconName::LoaderCircle)).with_animation(
        ElementId::CodeLocation(*std::panic::Location::caller()),
        Animation::new(std::time::Duration::from_secs(2)).repeat(),
        |el, delta| el.transform(Transformation::rotate(percentage(delta))),
    )
}
