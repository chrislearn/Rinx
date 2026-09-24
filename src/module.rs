//! Rinx as an OctoSense app module: the same client, seated in the host's pane.
//!
//! OctoSense links trusted native apps as `AppModule`s. The host owns the
//! window, safe area and theme; Rinx supplies its window-less `RinxContent`.
//! Matrix state is process-wide, so one instance runs at a time.
use makepad_app_module::{
    AppModule, ExecOutcome, InstanceHandles, InstanceParts, OpenSchema, ServiceExecutor, ValidatedOpen,
    makepad_ai_services::wire::{ServiceCall, ServiceManifest, ToolResult},
};
use makepad_widgets::*;
use std::sync::atomic::{AtomicBool, Ordering};

static INSTANCE_ACTIVE: AtomicBool = AtomicBool::new(false);

/// Whether Rinx is running inside an OctoSense host.
pub fn is_hosted() -> bool { INSTANCE_ACTIVE.load(Ordering::Acquire) }

script_mod! {
    use mod.prelude.widgets.*
    mod.widgets.RinxModuleView = set_type_default() do #(RinxModuleView::register_widget(vm)) {
        width: Fill height: Fill flow: Overlay
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct RinxModuleView {
    #[deref] view: View,
    #[rust] app: Option<crate::app::App>,
}

impl RinxModuleView {
    fn close(&mut self, cx: &mut Cx) {
        if let Some(mut app) = self.app.take() {
            app.close_embedded(cx);
            self.view.children.clear();
            INSTANCE_ACTIVE.store(false, Ordering::Release);
        }
    }
}

impl Widget for RinxModuleView {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        if matches!(event, Event::Shutdown) { self.close(cx); return; }
        if let Some(app) = self.app.as_mut() {
            AppMain::handle_event(app, cx, event);
        } else {
            self.view.handle_event(cx, event, scope);
        }
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        match self.app.as_mut() {
            Some(app) => app.draw_embedded(cx, &mut self.view, walk),
            None => self.view.draw_walk(cx, scope, walk),
        }
    }
}

pub struct RinxModule;
pub static RINX_MODULE: RinxModule = RinxModule;

impl AppModule for RinxModule {
    fn id(&self) -> &'static str { "rinx" }
    fn label(&self) -> &'static str { "Rinx" }
    fn capabilities(&self) -> &'static [&'static str] { &["net", "storage", "audio.output", "clipboard"] }
    fn open_schema(&self) -> OpenSchema { OpenSchema::new(1) }

    fn register(&self, vm: &mut ScriptVm) {
        crate::app::register_widgets(vm);
        script_mod(vm);
    }

    fn create(&self, vm: &mut ScriptVm, _open: ValidatedOpen, _handles: InstanceHandles) -> InstanceParts {
        let owns_runtime = INSTANCE_ACTIVE.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).is_ok();
        let value = script_eval!(vm, { mod.widgets.RinxModuleView {} });
        let root = WidgetRef::script_from_value(vm, value);
        if let Some(mut view) = root.borrow_mut::<RinxModuleView>() {
            if owns_runtime {
                let app = crate::app::App::create_embedded(vm);
                let content = app.content();
                view.view.children.push((live_id!(content), content.clone()));
                vm.cx_mut().widget_tree_insert_child_deep(view.widget_uid(), live_id!(content), content);
                vm.cx_mut().widget_tree_mark_dirty(view.widget_uid());
                view.app = Some(app);
            } else {
                let message = script_eval!(vm, {
                    use mod.prelude.widgets.*
                    Label { width: Fill draw_text.wrap: Words text: "Rinx is already open." }
                });
                view.view.children.push((live_id!(already_open), WidgetRef::script_from_value(vm, message)));
            }
        }
        let cleanup = root.clone();
        InstanceParts {
            root,
            executor: Box::new(RinxExecutor),
            shutdown: Box::new(move |vm| {
                if let Some(mut view) = cleanup.borrow_mut::<RinxModuleView>() { view.close(vm.cx_mut()); }
            }),
        }
    }
}

struct RinxExecutor;
impl ServiceExecutor for RinxExecutor {
    fn manifest(&self) -> ServiceManifest {
        ServiceManifest::new("rinx", "Rinx", "Matrix chats, Moments and articles.")
    }
    fn execute(&mut self, _cx: &mut Cx, call: &ServiceCall) -> ExecOutcome {
        ExecOutcome::Done(ToolResult::unavailable(&call.call_id, "Use the Rinx interface"))
    }
}
