use makepad_widgets::*;

use crate::sliding_sync::{submit_async_request, LoginByPassword, LoginRequest, MatrixRequest};

use super::homeserver::{password_identifier, LoginMethods};
use super::login_status_modal::{LoginStatusModalAction, LoginStatusModalWidgetExt};

script_mod! {
    use mod.prelude.widgets.*
    use mod.widgets.*

    mod.widgets.IMG_APP_LOGO = crate_resource("self://resources/robrix_logo_alpha.png")
    mod.widgets.ICON_EYE_OPEN   = crate_resource("self://resources/icons/eye_open.svg")
    mod.widgets.ICON_EYE_CLOSED = crate_resource("self://resources/icons/eye_closed.svg")

    mod.widgets.LoginScreen = set_type_default() do #(LoginScreen::register_widget(vm)) {
        ..mod.widgets.SolidView

        width: Fill, height: Fill,
        align: Align{x: 0.5, y: 0.5}
        show_bg: true,
        draw_bg +: {
            color: COLOR_SECONDARY
        }

        ScrollYView {
            width: Fill, height: Fill,
            flow: Down, // Required for vertical scrolling to work.
            align: Align{x: 0.5, y: 0.5}
            show_bg: true,
            draw_bg.color: (COLOR_SECONDARY)

            // allow the view to be scrollable but hide the actual scroll bar
            scroll_bars: {
                show_scroll_x: false, show_scroll_y: true,
                scroll_bar_y: {
                    bar_size: 0.0
                    min_handle_size: 0.0
                    drag_scrolling: true
                }
            }

            RoundedView {
                margin: Inset{top: 50, bottom: 50}
                width: Fill
                height: Fit
                align: Align{x: 0.5, y: 0.5}
                flow: Overlay,

                show_bg: true,
                draw_bg +: {
                    color: (COLOR_SECONDARY)
                    border_radius: 6.0
                }

                View {
                    width: Fill
                    height: Fit
                    flow: Down
                    align: Align{x: 0.5, y: 0.5}
                    spacing: 15.0

                    logo_image := Image {
                        fit: ImageFit.Smallest,
                        width: 80
                        src: (mod.widgets.IMG_APP_LOGO),
                    }

                    title := Label {
                        width: Fit, height: Fit
                        margin: Inset{ bottom: 5 }
                        padding: 0,
                        draw_text +: {
                            color: (COLOR_TEXT)
                            text_style: TITLE_TEXT {font_size: 16.0}
                        }
                        text: #(crate::i18n::tr("Sign in to Rinx")) i18n_text: "Sign in to Rinx"
                    }

                    View {
                        width: Fit height: Fit flow: Right spacing: 12
                        login_language_en := ButtonFlat {text: "English"}
                        login_language_zh := ButtonFlat {text: "简体中文"}
                    }

                    server_step := View {
                        width: 320, height: Fit, flow: Down, spacing: 14

                        Label {
                            width: Fill, height: Fit
                            draw_text +: {color: COLOR_TEXT, text_style: REGULAR_TEXT {font_size: 12}}
                            text: #(crate::i18n::tr("Choose your homeserver")) i18n_text: "Choose your homeserver"
                        }
                        homeserver_input := RobrixTextInput {
                            width: Fill, height: Fit
                            padding: 10
                            empty_text: "matrix.org"
                            autocapitalize: None
                            autocorrect: Disabled
                            content_type: Url
                            input_mode: Url
                        }
                        Label {
                            width: Fill, height: Fit
                            flow: Flow.Right{wrap: true}
                            draw_text +: {color: #8C8C8C, text_style: REGULAR_TEXT {font_size: 10}}
                            text: #(crate::i18n::tr("Enter a Matrix server name or homeserver URL. Leave blank for matrix.org.")) i18n_text: "Enter a Matrix server name or homeserver URL. Leave blank for matrix.org."
                        }
                        continue_server_button := RobrixIconButton {
                            width: Fill, height: 42, padding: 10
                            align: Align{x: 0.5, y: 0.5}
                            text: #(crate::i18n::tr("Continue")) i18n_text: "Continue"
                        }
                        server_status := Label {
                            width: Fill, height: Fit
                            flow: Flow.Right{wrap: true}
                            draw_text +: {color: COLOR_TEXT, text_style: REGULAR_TEXT {font_size: 10}}
                            text: ""
                        }
                    }

                    method_step := View {
                        visible: false
                        width: 320, height: Fit, flow: Down, spacing: 14

                        selected_server := Label {
                            width: Fill, height: Fit
                            flow: Flow.Right{wrap: true}
                            draw_text +: {color: COLOR_TEXT, text_style: REGULAR_TEXT {font_size: 12}}
                            text: ""
                        }
                        change_server_button := ButtonFlat {
                            width: Fit, height: Fit
                            text: #(crate::i18n::tr("Change server")) i18n_text: "Change server"
                        }
                        method_status := Label {
                            width: Fill, height: Fit
                            flow: Flow.Right{wrap: true}
                            draw_text +: {color: COLOR_TEXT, text_style: REGULAR_TEXT {font_size: 10}}
                            text: ""
                        }
                        provider_list_container := View {
                            visible: false
                            width: Fill, height: Fit
                            provider_list := PortalList {
                                width: Fill, height: 48
                                Provider := View {
                                    width: Fill, height: 48
                                    provider_button := RobrixIconButton {
                                        width: Fill, height: 42, padding: 10
                                        align: Align{x: 0.5, y: 0.5}
                                        text: ""
                                    }
                                }
                            }
                        }
                        browser_login_button := RobrixIconButton {
                            visible: false
                            width: Fill, height: 42, padding: 10
                            align: Align{x: 0.5, y: 0.5}
                            text: #(crate::i18n::tr("Continue with single sign-on")) i18n_text: "Continue with single sign-on"
                        }
                        password_option_button := ButtonFlat {
                            visible: false
                            width: Fit, height: Fit
                            text: #(crate::i18n::tr("Sign in with a password instead")) i18n_text: "Sign in with a password instead"
                        }
                        password_form := View {
                            visible: false
                            width: Fill, height: Fit, flow: Down, spacing: 12

                            user_id_input := RobrixTextInput {
                                width: Fill, height: Fit
                                padding: 10
                                empty_text: #(crate::i18n::tr("Matrix ID, username or email")) i18n_empty_text: "Matrix ID, username or email"
                                autocapitalize: None
                                autocorrect: Disabled
                                content_type: Username
                            }
                            View {
                                width: Fill, height: Fit
                                flow: Overlay
                                align: Align{x: 1.0, y: 0.5}
                                password_input := RobrixTextInput {
                                    width: Fill, height: Fit
                                    padding: Inset{top: 10, bottom: 10, left: 10, right: 38}
                                    empty_text: #(crate::i18n::tr("Password")) i18n_empty_text: "Password"
                                    is_password: true
                                    autocapitalize: None
                                    autocorrect: Disabled
                                    content_type: Password
                                }
                                View {
                                    width: 38, height: Fill
                                    align: Align{x: 0.5, y: 0.5}
                                    show_password_button := RobrixNeutralIconButton {
                                        width: Fit, height: Fit, padding: 5, spacing: 0, margin: 0
                                        draw_icon +: {svg: (mod.widgets.ICON_EYE_CLOSED), color: #8C8C8C}
                                        icon_walk: Walk{width: 18, height: 18, margin: 0}
                                        text: ""
                                    }
                                    hide_password_button := RobrixNeutralIconButton {
                                        visible: false
                                        width: Fit, height: Fit, padding: 5, spacing: 0, margin: 0
                                        draw_icon +: {svg: (mod.widgets.ICON_EYE_OPEN), color: #8C8C8C}
                                        icon_walk: Walk{width: 18, height: 18, margin: 0}
                                        text: ""
                                    }
                                }
                            }
                            login_button := RobrixIconButton {
                                width: Fill, height: 42, padding: 10
                                align: Align{x: 0.5, y: 0.5}
                                text: #(crate::i18n::tr("Sign in with password")) i18n_text: "Sign in with password"
                            }
                        }
                    }
                }

                // The modal that pops up to display login status messages,
                // such as when the user is logging in or when there is an error.
                login_status_modal := Modal {
                    can_dismiss: false,
                    content := mod.widgets.LoginStatusModal {}
                }
            }
        }
    }
}

#[derive(Script, ScriptHook, Widget)]
pub struct LoginScreen {
    #[source] source: ScriptObjectRef,
    #[deref] view: View,
    /// Whether the password field is currently showing plaintext.
    #[rust] password_visible: bool,
    #[rust] sso_pending: bool,
    #[rust] login_pending: bool,
    #[rust] discovery_generation: u64,
    #[rust] discovery_pending: bool,
    #[rust] methods: Option<LoginMethods>,
    #[rust] password_form_open: bool,

}


impl Widget for LoginScreen {
    fn handle_event(&mut self, cx: &mut Cx, event: &Event, scope: &mut Scope) {
        self.view.handle_event(cx, event, scope);
        self.match_event(cx, event);
    }

    fn draw_walk(&mut self, cx: &mut Cx2d, scope: &mut Scope, walk: Walk) -> DrawStep {
        while let Some(item) = self.view.draw_walk(cx, scope, walk).step() {
            if let Some(mut list) = item.borrow_mut::<PortalList>() {
                let providers = self.methods.as_ref().map(|m| m.providers.as_slice()).unwrap_or_default();
                list.set_item_range(cx, 0, providers.len());
                while let Some(index) = list.next_visible_item(cx) {
                    if let Some(provider) = providers.get(index) {
                        let row = list.item(cx, index, id!(Provider));
                        row.button(cx, ids!(provider_button)).set_text(cx, &crate::i18n::format(
                            "Sign in with {provider}",
                            &[("provider", provider.name.clone())],
                        ));
                        row.draw_all(cx, scope);
                    }
                }
            }
        }
        DrawStep::done()
    }
}

impl LoginScreen {
    fn show_status(&mut self, cx: &mut Cx, title: &str, status: &str, button: &str, enabled: bool) {
        let content = self.view.login_status_modal(cx, ids!(login_status_modal.content));
        content.set_title(cx, title);
        content.set_status(cx, status);
        content.button_ref(cx).set_text(cx, button);
        content.button_ref(cx).set_enabled(cx, enabled);
        self.view.modal(cx, ids!(login_status_modal)).open(cx);
        self.redraw(cx);
    }

    fn reset_server(&mut self, cx: &mut Cx) {
        self.discovery_generation += 1;
        self.discovery_pending = false;
        self.methods = None;
        self.password_form_open = false;
        self.view.view(cx, ids!(server_step)).set_visible(cx, true);
        self.view.view(cx, ids!(method_step)).set_visible(cx, false);
        self.view.label(cx, ids!(server_status)).set_text(cx, "");
        self.view.text_input(cx, ids!(password_input)).set_text(cx, "");
        if self.password_visible {
            self.password_visible = false;
            self.view.text_input(cx, ids!(password_input)).toggle_is_password(cx);
            self.view.button(cx, ids!(show_password_button)).set_visible(cx, true);
            self.view.button(cx, ids!(hide_password_button)).set_visible(cx, false);
        }
    }

    fn check_server(&mut self, cx: &mut Cx, server: String) {
        self.discovery_generation += 1;
        let generation = self.discovery_generation;
        self.discovery_pending = true;
        self.view.label(cx, ids!(server_status)).set_text(cx, crate::i18n::tr("Checking homeserver and sign-in methods…"));
        crate::sliding_sync::spawn_async_task(async move {
            let result = super::homeserver::discover("", &server).await.map_err(|e| e.to_string());
            Cx::post_action(LoginAction::ServerDiscovered { generation, result });
        });
    }

    fn show_methods(&mut self, cx: &mut Cx, methods: LoginMethods) {
        let has_sso = methods.sso;
        let has_password = methods.password;
        let has_providers = !methods.providers.is_empty();
        self.view.label(cx, ids!(selected_server)).set_text(cx, &methods.homeserver);
        self.view.label(cx, ids!(method_status)).set_text(cx, if has_sso && has_providers {
            crate::i18n::tr("Choose a sign-in provider. Your server handles authentication in the browser.")
        } else if has_sso {
            crate::i18n::tr("Your server handles sign-in in the browser.")
        } else if has_password {
            crate::i18n::tr("This server supports password sign-in.")
        } else {
            crate::i18n::tr("This server did not advertise SSO or password sign-in.")
        });
        self.view.view(cx, ids!(provider_list_container)).set_visible(cx, has_sso && has_providers);
        let height = (methods.providers.len().min(4).max(1) * 48) as f64;
        let mut list = self.view.portal_list(cx, ids!(provider_list));
        script_apply_eval!(cx, list, {height: #(height)});
        self.view.button(cx, ids!(browser_login_button)).set_visible(cx, has_sso && !has_providers);
        self.view.button(cx, ids!(password_option_button)).set_visible(cx, has_sso && has_password);
        self.view.button(cx, ids!(password_option_button)).set_text(cx, crate::i18n::tr("Sign in with a password instead"));
        self.password_form_open = has_password && !has_sso;
        self.view.view(cx, ids!(password_form)).set_visible(cx, self.password_form_open);
        self.view.view(cx, ids!(server_step)).set_visible(cx, false);
        self.view.view(cx, ids!(method_step)).set_visible(cx, true);
        self.methods = Some(methods);
        self.redraw(cx);
    }

    fn start_sso(&mut self, cx: &mut Cx, provider_id: Option<String>) {
        let Some(methods) = self.methods.as_ref().filter(|m| m.sso) else { return };
        self.login_pending = true;
        self.sso_pending = true;
        let destination = methods.homeserver.clone();
        self.show_status(cx, crate::i18n::tr("Connecting to your server"), &crate::i18n::format("Checking browser sign-in for {destination}…", &[("destination", destination.clone())]), crate::i18n::tr("Cancel"), true);
        submit_async_request(MatrixRequest::SpawnSSOServer { homeserver_url: destination, provider_id });
    }
}

impl MatchEvent for LoginScreen {
    fn handle_actions(&mut self, cx: &mut Cx, actions: &Actions) {
        for (path, language) in [(ids!(login_language_en), crate::i18n::Language::English), (ids!(login_language_zh), crate::i18n::Language::Chinese)] {
            if self.view.button(cx, path).clicked(actions) {
                if let Err(error) = crate::i18n::set_language(cx, language) {
                    crate::shared::popup_list::enqueue_popup_notification(crate::i18n::format("Could not save language: {error}", &[("error", error.to_string())]), crate::shared::popup_list::PopupKind::Error, Some(5.0));
                }
            }
        }
        let user_input = self.view.text_input(cx, ids!(user_id_input));
        let password_input = self.view.text_input(cx, ids!(password_input));
        let server_input = self.view.text_input(cx, ids!(homeserver_input));
        let modal = self.view.modal(cx, ids!(login_status_modal));
        let server = server_input.text().trim().to_owned();

        let show = self.view.button(cx, ids!(show_password_button));
        let hide = self.view.button(cx, ids!(hide_password_button));
        if show.clicked(actions) || hide.clicked(actions) {
            self.password_visible = !self.password_visible;
            password_input.toggle_is_password(cx);
            show.set_visible(cx, !self.password_visible);
            hide.set_visible(cx, self.password_visible);
            password_input.set_key_focus(cx);
        }
        if server_input.changed(actions).is_some() {
            self.reset_server(cx);
        }
        if self.view.button(cx, ids!(change_server_button)).clicked(actions) {
            self.reset_server(cx);
            server_input.set_key_focus(cx);
        }
        if !self.login_pending && !self.discovery_pending && (
            self.view.button(cx, ids!(continue_server_button)).clicked(actions)
            || server_input.returned(actions).is_some()
        ) {
            self.check_server(cx, server.clone());
        }
        if !self.login_pending && self.password_form_open && self.methods.as_ref().is_some_and(|m| m.password) && (
            self.view.button(cx, ids!(login_button)).clicked(actions)
            || user_input.returned(actions).is_some()
            || password_input.returned(actions).is_some()
        ) {
            let user = user_input.text().trim().to_owned();
            let destination = self.methods.as_ref().map(|m| m.homeserver.clone()).ok_or_else(|| anyhow::anyhow!(crate::i18n::tr("Choose a server first."))).and_then(|server| {
                password_identifier(&user)?;
                if password_input.text().is_empty() { anyhow::bail!(crate::i18n::tr("Enter your password.")); }
                Ok(server)
            });
            match destination {
                Ok(destination) => {
                    self.login_pending = true;
                    self.show_status(cx, crate::i18n::tr("Signing in"), &crate::i18n::format("Connecting to {destination}…", &[("destination", (destination).to_string())]), crate::i18n::tr("Please wait…"), false);
                    submit_async_request(MatrixRequest::Login(LoginRequest::LoginByPassword(LoginByPassword {
                        user_id: user.clone(), password: password_input.text(), homeserver: Some(destination),
                    })));
                }
                Err(error) => self.show_status(cx, crate::i18n::tr("Check sign-in details"), &error.to_string(), crate::i18n::tr("Okay"), true),
            }
        }
        if self.view.button(cx, ids!(password_option_button)).clicked(actions) {
            self.password_form_open = !self.password_form_open;
            self.view.view(cx, ids!(password_form)).set_visible(cx, self.password_form_open);
            self.view.label(cx, ids!(method_status)).set_text(cx, if self.password_form_open {
                crate::i18n::tr("This server supports password sign-in.")
            } else if self.methods.as_ref().is_some_and(|m| m.providers.is_empty()) {
                crate::i18n::tr("Your server handles sign-in in the browser.")
            } else {
                crate::i18n::tr("Choose a sign-in provider. Your server handles authentication in the browser.")
            });
            self.view.button(cx, ids!(password_option_button)).set_text(cx, if self.password_form_open {
                crate::i18n::tr("Hide password sign-in")
            } else {
                crate::i18n::tr("Sign in with a password instead")
            });
        }
        if !self.login_pending && self.view.button(cx, ids!(browser_login_button)).clicked(actions) {
            self.start_sso(cx, None);
        }
        if !self.login_pending {
            let selected_provider = self.view.portal_list(cx, ids!(provider_list))
                .items_with_actions(actions)
                .into_iter()
                .find_map(|(index, row)| {
                    if row.button(cx, ids!(provider_button)).clicked(actions) {
                        self.methods.as_ref().and_then(|m| m.providers.get(index)).map(|p| p.id.clone())
                    } else { None }
                });
            if let Some(provider_id) = selected_provider {
                self.start_sso(cx, Some(provider_id));
            }
        }

        for action in actions {
            if let LoginStatusModalAction::Close = action.as_widget_action().cast() {
                if self.sso_pending {
                    submit_async_request(MatrixRequest::CancelSsoLogin);
                    self.show_status(cx, crate::i18n::tr("Cancelling sign-in"), crate::i18n::tr("Closing the sign-in connection…"), crate::i18n::tr("Please wait…"), false);
                } else if !self.login_pending {
                    modal.close(cx);
                }
            }
            match action.downcast_ref() {
                Some(LoginAction::ServerDiscovered { generation, result }) if *generation == self.discovery_generation => {
                    self.discovery_pending = false;
                    match result {
                        Ok(methods) => self.show_methods(cx, methods.clone()),
                        Err(error) => self.view.label(cx, ids!(server_status)).set_text(cx, &crate::i18n::format("Could not check this server: {error}", &[("error", error.clone())])),
                    }
                }
                Some(LoginAction::CliAutoLogin { user_id, homeserver }) => {
                    self.login_pending = true;
                    user_input.set_text(cx, user_id);
                    password_input.set_text(cx, "");
                    server_input.set_text(cx, homeserver.as_deref().unwrap_or_default());
                    self.reset_server(cx);
                    self.show_status(cx, crate::i18n::tr("Signing in"), crate::i18n::tr("Connecting to your account…"), crate::i18n::tr("Please wait…"), false);
                }
                Some(LoginAction::Status { title, status }) => {
                    self.login_pending = true;
                    self.show_status(cx, title, status, if self.sso_pending { crate::i18n::tr("Cancel") } else { crate::i18n::tr("Please wait…") }, self.sso_pending);
                }
                Some(LoginAction::LoginSuccess) => {
                    self.login_pending = false;
                    self.sso_pending = false;
                    user_input.set_text(cx, "");
                    password_input.set_text(cx, "");
                    server_input.set_text(cx, "");
                    self.reset_server(cx);
                    modal.close(cx);
                }
                Some(LoginAction::LoginFailure(error)) => {
                    self.login_pending = false;
                    self.sso_pending = false;
                    self.show_status(cx, crate::i18n::tr("Sign-in failed"), error, crate::i18n::tr("Okay"), true);
                }
                Some(LoginAction::SsoPending(pending)) => self.sso_pending = *pending,
                Some(LoginAction::Cancelled) => {
                    self.login_pending = false;
                    self.sso_pending = false;
                    modal.close(cx);
                }
                _ => {}
            }
        }
        self.view.button(cx, ids!(login_button)).set_enabled(cx, !self.login_pending);
        self.view.button(cx, ids!(browser_login_button)).set_enabled(cx, !self.login_pending);
        self.view.button(cx, ids!(continue_server_button)).set_enabled(cx, !self.login_pending && !self.discovery_pending);
        self.redraw(cx);
    }
}

/// Actions sent to or from the login screen. Never include a password or token.
#[derive(Clone, Default, Debug)]
pub enum LoginAction {
    LoginSuccess,
    LoginFailure(String),
    Status { title: String, status: String },
    CliAutoLogin { user_id: String, homeserver: Option<String> },
    SsoPending(bool),
    Cancelled,
    ServerDiscovered { generation: u64, result: Result<LoginMethods, String> },
    #[default]
    None,
}
