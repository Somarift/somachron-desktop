use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    ActiveTheme, Disableable, IconName, StyledExt, WindowExt,
    button::{Button, ButtonVariants},
    dialog::Dialog,
    input::{Input, InputState},
    label::Label,
    v_flex,
};

pub trait CreateSpaceDialog: Sized {
    fn create_space(
        &mut self,
        name: SharedString,
        description: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    );

    fn is_loading(&self) -> bool;
}

pub fn trigger<T: CreateSpaceDialog + 'static>(entity: WeakEntity<T>) -> Button {
    Button::new("create_space")
        .primary()
        .icon(IconName::Plus)
        .label("Create space")
        .on_click(move |_ev, window, cx| {
            let entity = entity.clone();

            let name_input_state = cx.new(|cx| InputState::new(window, cx).placeholder("Name"));
            let description_input_state = cx.new(|cx| InputState::new(window, cx).placeholder("Description"));

            window.open_dialog(cx, move |dialog, _window, cx| {
                comp(
                    dialog,
                    entity.clone(),
                    name_input_state.clone(),
                    description_input_state.clone(),
                    cx,
                )
            });
        })
}

fn comp<T: CreateSpaceDialog + 'static>(
    dialog: Dialog,
    entity: WeakEntity<T>,
    name_input_state: Entity<InputState>,
    description_input_state: Entity<InputState>,
    cx: &mut App,
) -> Dialog {
    let (name_is_empty, name_is_invalid) =
        name_input_state.read_with(cx, |this, _cx| (this.value().is_empty(), this.value().len() > 64));
    let is_creating_space = entity.read_with(cx, |this, _cx| this.is_loading()).unwrap_or_default();

    let _entity = entity.clone();

    dialog
        .alert()
        .keyboard(false)
        .overlay_closable(false)
        .rounded_lg()
        .title("Create new space")
        .v_flex()
        .max_h_128()
        .child(
            v_flex()
                .gap_2()
                .child(Label::new("Your shareable space (private by default)"))
                .child(Input::new(&name_input_state).cleanable(true).line_clamp(1).map(|this| {
                    if name_is_invalid {
                        this.border_1().border_color(cx.theme().danger)
                    } else {
                        this
                    }
                }))
                .when(name_is_invalid, |el| {
                    el.child(
                        div()
                            .text_xs()
                            .child("Name cannot be more than 64 chars")
                            .text_color(cx.theme().danger),
                    )
                })
                .child(Input::new(&description_input_state).cleanable(true).line_clamp(1)),
        )
        .footer(move |_, _, _, _| {
            let name_input_state = name_input_state.clone();
            let description_input_state = description_input_state.clone();
            let entity = _entity.clone();

            let cancel = Button::new("cancel_create_space")
                .label("Cancel")
                .disabled(is_creating_space)
                .on_click(|_, window, cx| {
                    window.close_dialog(cx);
                });

            let ok = Button::new("ok_create_space")
                .primary()
                .label("Create")
                .disabled(name_is_empty)
                .loading_icon(IconName::LoaderCircle)
                .loading(is_creating_space)
                .on_click(move |_ev, window, cx| {
                    let name = name_input_state.clone().read_with(cx, |state, _cx| state.value());
                    let description = description_input_state
                        .clone()
                        .read_with(cx, |state, _cx| state.value());

                    let _ = entity.update(cx, |this, cx| {
                        this.create_space(name, description, window, cx);
                    });
                });

            vec![cancel, ok]
        })
}
