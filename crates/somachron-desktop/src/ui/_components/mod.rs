use gpui::*;
use gpui_component::{Icon, IconName};

pub mod app_icon;
pub mod create_folder_dialog;
pub mod create_space_dialog;
pub mod delete_dialog;
pub mod select_space_dialog;

pub fn loading_icon(f: impl FnOnce(Icon) -> Icon) -> impl IntoElement {
    f(Icon::new(IconName::LoaderCircle)).with_animation(
        ElementId::CodeLocation(*std::panic::Location::caller()),
        Animation::new(std::time::Duration::from_secs(2)).repeat(),
        |el, delta| el.transform(Transformation::rotate(percentage(delta))),
    )
}
