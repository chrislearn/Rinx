//! Standalone selection UI. Hosted instances display the host's ownership.
use makepad_widgets::*;
use crate::theme::{self, Accent, Appearance, Selection};

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*
    mod.widgets.AppearanceSettings = #(AppearanceSettings::register_widget(vm)) {
        width: Fill height: Fit flow: Down spacing: 8 padding: Inset{top: 12 bottom: 16}
        RinxLabel {text: #(crate::i18n::tr("App appearance")) i18n_text: "App appearance" draw_text.text_style: theme.font_bold{font_size: 13}}
        choices := View {width: Fill height: Fit flow: Flow.Right{wrap: true} spacing: 12
            mode := DropDown {width: 150 height: 44 labels: [#(crate::i18n::tr("Light")), #(crate::i18n::tr("Dark"))] i18n_labels: ["Light", "Dark"]}
            accent := DropDown {width: 150 height: 44 labels: [#(crate::i18n::tr("Teal")), #(crate::i18n::tr("Violet"))] i18n_labels: ["Teal", "Violet"]}
        }
        host_note := RinxHint {visible: false width: Fill text: #(crate::i18n::tr("Appearance is managed by OctoSense")) i18n_text: "Appearance is managed by OctoSense"}
        error := RinxHint {visible: false width: Fill}
    }
}

#[derive(Script, Widget)]
pub struct AppearanceSettings {
    #[deref]
    view: View,
    #[rust]
    selection: Option<Selection>,
}
impl ScriptHook for AppearanceSettings {
    fn on_after_apply(
        &mut self,
        vm: &mut ScriptVm,
        _apply: &Apply,
        _scope: &mut Scope,
        _value: ScriptValue,
    ) {
        self.selection = theme::selection_for_vm(vm);
        let cx = vm.cx_mut();
        self.view
            .view(cx, ids!(choices))
            .set_visible(cx, self.selection.is_some());
        self.view
            .label(cx, ids!(host_note))
            .set_visible(cx, self.selection.is_none());
        if let Some(s) = self.selection {
            self.view
                .drop_down(cx, ids!(mode))
                .set_selected_item(cx, usize::from(s.appearance == Appearance::Dark));
            self.view
                .drop_down(cx, ids!(accent))
                .set_selected_item(cx, usize::from(s.accent == Accent::Violet));
        }
    }
}
impl Widget for AppearanceSettings {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        if let Event::Actions(actions) = event {
            let Some(mut selection) = self.selection else {
                return;
            };
            let mode = self.view.drop_down(cx, ids!(mode)).changed(actions);
            let accent = self.view.drop_down(cx, ids!(accent)).changed(actions);
            if let Some(index) = mode {
                selection.appearance = if index == 1 {
                    Appearance::Dark
                } else {
                    Appearance::Light
                };
            }
            if let Some(index) = accent {
                selection.accent = if index == 1 {
                    Accent::Violet
                } else {
                    Accent::Teal
                };
            }
            if mode.is_some() || accent.is_some() {
                match theme::select(cx, selection) {
                    Ok(()) => {
                        self.selection = Some(selection);
                        self.view.label(cx, ids!(error)).set_visible(cx, false);
                    }
                    Err(error) => {
                        if let Some(previous) = self.selection {
                            self.view.drop_down(cx, ids!(mode)).set_selected_item(
                                cx,
                                usize::from(previous.appearance == Appearance::Dark),
                            );
                            self.view.drop_down(cx, ids!(accent)).set_selected_item(
                                cx,
                                usize::from(previous.accent == Accent::Violet),
                            );
                        }
                        self.view.label(cx, ids!(error)).set_text(cx, &error);
                        self.view.label(cx, ids!(error)).set_visible(cx, true);
                    }
                }
            }
        }
    }
    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        self.view.draw_walk(cx, scope, walk)
    }
}
