use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    ActiveTheme, Disableable, Icon, IconName, StyledExt, WindowExt,
    button::{Button, ButtonVariants},
    dialog::Dialog,
    input::{Input, InputState},
    label::Label,
    v_flex,
};

pub trait CreateFolderDialog: Sized {
    fn create_folder(&mut self, name: SharedString, window: &mut Window, cx: &mut Context<Self>);

    fn current_path(&self) -> SharedString;

    fn is_loading(&self) -> bool;
}

pub fn trigger<T: CreateFolderDialog + 'static>(entity: WeakEntity<T>) -> Button {
    Button::new("create_folder")
        .primary()
        .icon(Icon::empty().path("icons/folder-plus.svg"))
        .label("Create folder")
        .on_click(move |_ev, window, cx| {
            let entity = entity.clone();

            let name_input_state =
                cx.new(|cx| InputState::new(window, cx).placeholder("Folder name"));

            window.open_dialog(cx, move |dialog, _window, cx| {
                comp(dialog, entity.clone(), name_input_state.clone(), cx)
            });
        })
}

fn comp<T: CreateFolderDialog + 'static>(
    dialog: Dialog,
    entity: WeakEntity<T>,
    name_input_state: Entity<InputState>,
    cx: &mut App,
) -> Dialog {
    let (name_is_empty, name_is_invalid) = name_input_state.read_with(cx, |this, _cx| {
        (this.value().is_empty(), this.value().len() > 64)
    });

    let (is_loading, current_path) = entity
        .read_with(cx, |this, _cx| (this.is_loading(), this.current_path()))
        .unwrap_or_default();

    let _entity = entity.clone();

    dialog
        .alert()
        .keyboard(false)
        .overlay_closable(false)
        .rounded_lg()
        .title("Create new folder")
        .v_flex()
        .max_h_128()
        .child(
            v_flex()
                .gap_2()
                .child(Label::new(format!("Path: {current_path}")))
                .child(
                    Input::new(&name_input_state)
                        .cleanable(true)
                        .line_clamp(1)
                        .map(|this| {
                            if name_is_invalid {
                                this.border_1().border_color(cx.theme().danger)
                            } else {
                                this
                            }
                        }),
                )
                .when(name_is_invalid, |el| {
                    el.child(
                        div()
                            .text_xs()
                            .child("Name cannot be more than 64 chars")
                            .text_color(cx.theme().danger),
                    )
                }),
        )
        .footer(move |_, _, _, _| {
            let name_input_state = name_input_state.clone();
            let entity = _entity.clone();

            let cancel = Button::new("cancel_create_folder")
                .label("Cancel")
                .disabled(is_loading)
                .on_click(|_, window, cx| {
                    window.close_dialog(cx);
                });

            let ok = Button::new("ok_create_folder")
                .primary()
                .label("Create")
                .disabled(name_is_empty)
                .loading_icon(IconName::LoaderCircle)
                .loading(is_loading)
                .on_click(move |_ev, window, cx| {
                    let name = name_input_state
                        .clone()
                        .read_with(cx, |state, _cx| state.value());

                    let _ = entity.update(cx, |this, cx| {
                        this.create_folder(name, window, cx);
                    });
                });

            vec![cancel, ok]
        })
}
