use warpui::{
    elements::{Container, Empty},
    AppContext, Element, Entity, ModelHandle, TypedActionView, View, ViewContext,
};

use crate::pane_group::{
    focus_state::PaneFocusHandle,
    pane::view,
    BackingView, PaneConfiguration, PaneEvent,
};

#[derive(Clone, Debug)]
pub enum WebViewAction {}

pub struct WebViewView {
    pane_configuration: ModelHandle<PaneConfiguration>,
    focus_handle: Option<PaneFocusHandle>,
    title: String,
    url: String,
}

impl WebViewView {
    pub fn new(title: String, url: String, ctx: &mut ViewContext<Self>) -> Self {
        let pane_configuration = ctx.add_model(|_ctx| PaneConfiguration::new(&title));
        Self {
            pane_configuration,
            focus_handle: None,
            title,
            url,
        }
    }

    pub fn pane_configuration(&self) -> ModelHandle<PaneConfiguration> {
        self.pane_configuration.clone()
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}

impl Entity for WebViewView {
    type Event = PaneEvent;
}

impl View for WebViewView {
    fn render(&self, _ctx: &AppContext) -> Box<dyn Element> {
        // The actual WKWebView is overlaid natively by the pane —
        // this Rust view just provides a background/placeholder.
        Container::new(Empty::new().finish()).finish()
    }

    fn ui_name() -> &'static str {
        "WebViewView"
    }
}

impl TypedActionView for WebViewView {
    type Action = WebViewAction;

    fn handle_action(&mut self, _action: &Self::Action, _ctx: &mut ViewContext<Self>) {
        // No actions yet
    }
}

impl BackingView for WebViewView {
    type PaneHeaderOverflowMenuAction = ();
    type CustomAction = ();
    type AssociatedData = ();

    fn handle_pane_header_overflow_menu_action(
        &mut self,
        _action: &Self::PaneHeaderOverflowMenuAction,
        _ctx: &mut ViewContext<Self>,
    ) {
        // No overflow menu actions for WebView panes
    }

    fn close(&mut self, ctx: &mut ViewContext<Self>) {
        ctx.emit(PaneEvent::Close);
    }

    fn focus_contents(&mut self, _ctx: &mut ViewContext<Self>) {
        // Focus handling is managed by the focus_handle
        // The actual focus is handled by the pane system
    }

    fn render_header_content(
        &self,
        _ctx: &view::HeaderRenderContext<'_>,
        _app: &AppContext,
    ) -> view::HeaderContent {
        view::HeaderContent::simple(&self.title)
    }

    fn set_focus_handle(&mut self, focus_handle: PaneFocusHandle, _ctx: &mut ViewContext<Self>) {
        self.focus_handle = Some(focus_handle);
    }
}
