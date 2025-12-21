use gpui::{prelude::FluentBuilder, *};
use gpui_component::{
    ActiveTheme, Disableable, Icon, IconName, Sizable, StyledExt, WindowExt,
    button::{Button, ButtonVariants},
    dialog::Dialog,
    label::Label,
    v_flex,
};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum DeleteType {
    File(Vec<(Uuid, SharedString)>),
    Folder((Uuid, SharedString)),
}

impl DeleteType {
    pub fn get_type(&self) -> &'static str {
        match self {
            DeleteType::File(_) => "file(s)",
            DeleteType::Folder(_) => "folder(s)",
        }
    }

    fn get_icon(&self) -> Icon {
        match self {
            DeleteType::File(_) => Icon::new(IconName::File),
            DeleteType::Folder(_) => Icon::new(IconName::Folder),
        }
    }
}

pub trait DeleteDialog: Sized {
    fn delete(&mut self, fs_id: DeleteType, window: &mut Window, cx: &mut Context<Self>);

    fn is_loading(&self) -> bool;
}

pub fn comp<T: DeleteDialog + 'static>(
    dialog: Dialog,
    entity: WeakEntity<T>,
    deletion_ty: DeleteType,
    cx: &mut App,
) -> Dialog {
    let is_loading = entity
        .read_with(cx, |this, _cx| this.is_loading())
        .unwrap_or_default();

    let deletion_type = deletion_ty.get_type();

    let _entity = entity.clone();

    dialog
        .alert()
        .keyboard(false)
        .overlay_closable(false)
        .rounded_lg()
        .title(format!("Delete {deletion_type}"))
        .v_flex()
        .max_h_128()
        .child(
            v_flex()
                .gap_2()
                .child(Label::new(format!(
                    "The following {deletion_type} will be deleted. Are you sure to continue ?",
                )))
                .map(|this| match deletion_ty.clone() {
                    DeleteType::File(uuids) => this.children(uuids.iter().map(|(_, name)| {
                        div()
                            .px_2()
                            .py_1()
                            .rounded_md()
                            .bg(cx.theme().sidebar)
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(deletion_ty.get_icon().small())
                            .text_sm()
                            .child(Label::new(name))
                    })),
                    DeleteType::Folder((_, name)) => this.child(Label::new(name)),
                }),
        )
        .footer(move |_, _, _, _| {
            let entity = _entity.clone();
            let del_ty = deletion_ty.clone();

            let cancel = Button::new("cancel_delete_fs")
                .label("Cancel")
                .disabled(is_loading)
                .on_click(|_, window, cx| {
                    window.close_dialog(cx);
                });

            let ok = Button::new("ok_delete_fs")
                .danger()
                .label("Delete")
                .loading_icon(IconName::LoaderCircle)
                .loading(is_loading)
                .on_click(move |_ev, window, cx| {
                    let _ = entity.clone().update(cx, |this, cx| {
                        this.delete(del_ty.clone(), window, cx);
                    });
                });

            vec![cancel, ok]
        })
}
