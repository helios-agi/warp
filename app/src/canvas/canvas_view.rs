use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::c_char;

use warpui::{
    elements::{
        Container, CrossAxisAlignment, Empty, Expanded, Flex, MainAxisSize,
        Padding, ParentElement as _,
    },
    ui_components::components::{UiComponent, UiComponentStyles},
    AppContext, Element, Entity, ModelHandle, SingletonEntity as _, TypedActionView, View,
    ViewContext,
};

use crate::appearance::Appearance;
use crate::pane_group::{
    focus_state::PaneFocusHandle,
    pane::view,
    BackingView, PaneConfiguration, PaneEvent,
};

use super::registry::resolve_canvas_view;
use super::types::{CanvasTab, CanvasTabId, CanvasViewKind};

#[derive(Clone, Debug)]
pub enum CanvasAction {
    OpenTab {
        view_key: String,
        label: String,
        metadata: HashMap<String, String>,
    },
    CloseTab {
        tab_id: CanvasTabId,
    },
    ActivateTab {
        tab_id: CanvasTabId,
    },
    PinTab {
        tab_id: CanvasTabId,
    },
    TogglePalette,
    PaletteSelect {
        view_key: String,
    },
}

pub struct CanvasView {
    pane_configuration: ModelHandle<PaneConfiguration>,
    focus_handle: Option<PaneFocusHandle>,
    tabs: Vec<CanvasTab>,
    active_tab_id: Option<CanvasTabId>,
    tab_counter: u64,
    pub palette_open: bool,
    palette_item_states: RefCell<Vec<warpui::elements::MouseStateHandle>>,
    #[cfg(target_os = "macos")]
    webviews: RefCell<HashMap<CanvasTabId, warpui::platform::mac::webview::HeliosWebView>>,
}

impl CanvasView {
    pub fn new(ctx: &mut ViewContext<Self>) -> Self {
        let pane_configuration = ctx.add_model(|_ctx| PaneConfiguration::new("Canvas"));
        Self {
            pane_configuration,
            focus_handle: None,
            tabs: Vec::new(),
            active_tab_id: None,
            tab_counter: 0,
            palette_open: false,
            palette_item_states: RefCell::new(Vec::new()),
            #[cfg(target_os = "macos")]
            webviews: RefCell::new(HashMap::new()),
        }
    }

    pub fn pane_configuration(&self) -> ModelHandle<PaneConfiguration> {
        self.pane_configuration.clone()
    }

    pub fn tabs(&self) -> &[CanvasTab] {
        &self.tabs
    }

    pub fn active_tab_id(&self) -> Option<&str> {
        self.active_tab_id.as_deref()
    }

    pub fn open_tab(
        &mut self,
        view_key: String,
        label: String,
        metadata: HashMap<String, String>,
        ctx: &mut ViewContext<Self>,
    ) {
        self.tab_counter += 1;
        let tab_id = format!("canvas-{}-{}", view_key, self.tab_counter);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        let tab = CanvasTab {
            id: tab_id.clone(),
            view_key: view_key.clone(),
            label,
            pinned: false,
            metadata: metadata.clone(),
            last_accessed: now,
        };
        self.tabs.push(tab);

        #[cfg(target_os = "macos")]
        self.hide_all_webviews();

        self.active_tab_id = Some(tab_id.clone());

        #[cfg(target_os = "macos")]
        if let Some(reg) = resolve_canvas_view(&view_key) {
            if let CanvasViewKind::WebView { filename } = &reg.kind {
                self.create_webview_for_tab(&tab_id, filename, ctx);
                self.inject_metadata_after_load(&tab_id, &view_key, &metadata);
            }
        }

        self.pane_configuration.update(ctx, |config, ctx| {
            let title = self.tabs.last().map(|t| t.label.as_str()).unwrap_or("Canvas");
            config.set_title(title, ctx);
        });

        ctx.notify();
    }

    pub fn close_tab(&mut self, tab_id: &str, ctx: &mut ViewContext<Self>) {
        if let Some(pos) = self.tabs.iter().position(|t| t.id == tab_id) {
            if self.tabs[pos].pinned {
                return;
            }
            self.tabs.remove(pos);

            #[cfg(target_os = "macos")]
            {
                self.webviews.borrow_mut().remove(tab_id);
            }

            if self.active_tab_id.as_deref() == Some(tab_id) {
                self.active_tab_id = self.tabs.last().map(|t| t.id.clone());
                #[cfg(target_os = "macos")]
                if let Some(ref new_active) = self.active_tab_id {
                    self.show_webview(new_active);
                }
            }
            ctx.notify();
        }
    }

    pub fn activate_tab(&mut self, tab_id: &str, ctx: &mut ViewContext<Self>) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == tab_id) {
            tab.last_accessed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);
            self.active_tab_id = Some(tab_id.to_string());

            #[cfg(target_os = "macos")]
            {
                self.hide_all_webviews();
                self.show_webview(tab_id);
            }

            ctx.notify();
        }
    }

    pub fn toggle_pin(&mut self, tab_id: &str, ctx: &mut ViewContext<Self>) {
        if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == tab_id) {
            tab.pinned = !tab.pinned;
            ctx.notify();
        }
    }

    #[cfg(target_os = "macos")]
    pub fn evaluate_js_on_tab(&self, tab_id: &str, js: &str) {
        if let Some(webview) = self.webviews.borrow().get(tab_id) {
            webview.eval_js(js);
        }
    }

    #[cfg(target_os = "macos")]
    pub fn evaluate_js_on_active(&self, js: &str) {
        if let Some(tab_id) = &self.active_tab_id {
            self.evaluate_js_on_tab(tab_id, js);
        }
    }

    #[cfg(target_os = "macos")]
    fn create_webview_for_tab(&self, tab_id: &str, filename: &str, ctx: &mut ViewContext<Self>) {
        use cocoa::appkit::NSView;
        use cocoa::foundation::NSRect;
        use warpui::platform::mac::content_view_from_platform_window;
        use warpui::platform::mac::webview::HeliosWebView;

        let url = crate::workspace::view::Workspace::webview_resource_url(filename);
        let window_id = ctx.window_id();
        if let Some(platform_window) = ctx.windows().platform_window(window_id) {
            if let Some(content_view) =
                content_view_from_platform_window(platform_window.as_ref())
            {
                let frame: NSRect = unsafe { NSView::bounds(content_view) };
                let webview =
                    HeliosWebView::new(frame, Some(&url), Some(canvas_ipc_callback), std::ptr::null_mut());
                webview.add_to_view(content_view);
                webview.set_autoresize();
                self.webviews
                    .borrow_mut()
                    .insert(tab_id.to_string(), webview);
            }
        }
    }

    #[cfg(target_os = "macos")]
    fn hide_all_webviews(&self) {
        use cocoa::base::NO;
        use objc::{msg_send, sel, sel_impl};
        for webview in self.webviews.borrow().values() {
            let native = webview.native_id();
            unsafe {
                let _: () = msg_send![native, setHidden: !NO];
            }
        }
    }

    #[cfg(target_os = "macos")]
    fn show_webview(&self, tab_id: &str) {
        use cocoa::base::NO;
        use objc::{msg_send, sel, sel_impl};
        if let Some(webview) = self.webviews.borrow().get(tab_id) {
            let native = webview.native_id();
            unsafe {
                let _: () = msg_send![native, setHidden: NO];
            }
        }
    }

    #[cfg(target_os = "macos")]
    fn inject_metadata_after_load(
        &self,
        tab_id: &str,
        view_key: &str,
        metadata: &HashMap<String, String>,
    ) {
        let js = match view_key {
            "interview" => metadata.get("questions").map(|questions| {
                let interview_json = format!(
                    r#"{{"id":"agent-interview-{}","title":"Agent Interview","questions":{}}}"#,
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis())
                        .unwrap_or(0),
                    questions
                );
                let escaped = escape_for_js_string(&interview_json);
                format!(
                    "window.__pendingInterviewData = '{}'; \
                     setTimeout(function() {{ \
                       if (window._heliosInterviewData) {{ \
                         window._heliosInterviewData(window.__pendingInterviewData); \
                       }} \
                     }}, 800);",
                    escaped
                )
            }),
            "design-deck" => metadata.get("slides").map(|slides| {
                let deck_json = format!(
                    r#"{{"id":"agent-deck-{}","title":"Agent Design Deck","slides":{}}}"#,
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_millis())
                        .unwrap_or(0),
                    slides
                );
                let escaped = escape_for_js_string(&deck_json);
                format!(
                    "window.__pendingDesignDeckData = '{}'; \
                     setTimeout(function() {{ \
                       if (window._heliosDesignDeckData) {{ \
                         window._heliosDesignDeckData(window.__pendingDesignDeckData); \
                       }} \
                     }}, 800);",
                    escaped
                )
            }),
            _ => None,
        };

        if let Some(js_code) = js {
            self.evaluate_js_on_tab(tab_id, &js_code);
        }
    }

    #[cfg(not(target_os = "macos"))]
    fn create_webview_for_tab(
        &self,
        _tab_id: &str,
        _filename: &str,
        _ctx: &mut ViewContext<Self>,
    ) {
    }

    fn render_tab_bar(&self, app: &AppContext) -> Box<dyn Element> {
        use warpui::elements::Text;
        use pathfinder_color::ColorU;

        let appearance = Appearance::as_ref(app);
        let font_family = appearance.monospace_font_family();

        if self.tabs.is_empty() {
            return Empty::new().finish();
        }

        let mut row = Flex::row().with_cross_axis_alignment(CrossAxisAlignment::Center);

        for tab in &self.tabs {
            let is_active = self.active_tab_id.as_deref() == Some(tab.id.as_str());

            let color = if is_active {
                ColorU::new(255, 255, 255, 255)
            } else {
                ColorU::new(128, 128, 128, 255)
            };

            let tab_element = Container::new(
                Text::new(tab.label.clone(), font_family, 12.)
                    .with_color(color)
                    .finish(),
            )
            .with_padding(Padding::uniform(6.).with_left(12.).with_right(12.))
            .finish();

            row = row.with_child(tab_element);
        }

        Container::new(row.with_main_axis_size(MainAxisSize::Max).finish())
            .with_padding_bottom(1.)
            .finish()
    }

    fn render_empty_state(&self, app: &AppContext) -> Box<dyn Element> {
        let appearance = Appearance::as_ref(app);
        let theme = appearance.theme();

        Flex::column()
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_children([
                Container::new(
                    appearance
                        .ui_builder()
                        .paragraph("Canvas")
                        .with_style(UiComponentStyles {
                            font_size: Some(18.),
                            ..Default::default()
                        })
                        .build()
                        .finish(),
                )
                .with_margin_bottom(8.)
                .finish(),
                appearance
                    .ui_builder()
                    .paragraph("No views open. Use the workspace menu to open a canvas view.")
                    .with_style(UiComponentStyles {
                        font_size: Some(13.),
                        font_color: Some(
                            theme.disabled_text_color(theme.background()).into_solid(),
                        ),
                        ..Default::default()
                    })
                    .build()
                    .finish(),
            ])
            .with_main_axis_size(MainAxisSize::Max)
            .finish()
    }

    fn render_palette(&self, app: &AppContext) -> Box<dyn Element> {
        use super::registry::all_canvas_views;
        use warpui::ui_components::button::{ButtonVariant, TextAndIcon, TextAndIconAlignment};
        use warpui::elements::MainAxisAlignment;
        use warpui::elements::ConstrainedBox;
        use pathfinder_geometry::vector::vec2f;
        use warp_core::ui::color::blend::Blend as _;

        let appearance = Appearance::as_ref(app);
        let theme = appearance.theme();
        let views = all_canvas_views();

        {
            let mut states = self.palette_item_states.borrow_mut();
            while states.len() < views.len() {
                states.push(Default::default());
            }
        }

        let mut list = Flex::column()
            .with_cross_axis_alignment(CrossAxisAlignment::Stretch);

        list = list.with_child(
            Container::new(
                appearance
                    .ui_builder()
                    .paragraph("Open Canvas View")
                    .with_style(UiComponentStyles {
                        font_size: Some(14.),
                        ..Default::default()
                    })
                    .build()
                    .finish(),
            )
            .with_padding(Padding::uniform(8.).with_bottom(12.))
            .finish(),
        );

        let states = self.palette_item_states.borrow();
        for (i, view_reg) in views.iter().enumerate() {
            let state = states[i].clone();
            let view_key = view_reg.key.clone();

            let item = appearance
                .ui_builder()
                .button(ButtonVariant::Text, state)
                .with_style(UiComponentStyles {
                    padding: Some(warpui::ui_components::components::Coords::uniform(6.)),
                    ..Default::default()
                })
                .with_hovered_styles(UiComponentStyles {
                    background: Some(
                        theme.background().blend(&theme.surface_overlay_1()).into(),
                    ),
                    ..Default::default()
                })
                .with_text_and_icon_label(TextAndIcon::new(
                    TextAndIconAlignment::IconFirst,
                    view_reg.label.clone(),
                    warp_core::ui::Icon::Terminal.to_warpui_icon(theme.foreground()),
                    MainAxisSize::Max,
                    MainAxisAlignment::Start,
                    vec2f(14., 14.),
                ))
                .build()
                .on_click(move |ctx, _, _| {
                    ctx.dispatch_typed_action(CanvasAction::PaletteSelect {
                        view_key: view_key.clone(),
                    });
                })
                .with_cursor(warpui::platform::Cursor::PointingHand)
                .finish();

            list = list.with_child(item);
        }

        Container::new(
            ConstrainedBox::new(list.finish())
                .with_max_width(320.)
                .finish(),
        )
        .with_padding(Padding::uniform(12.))
        .finish()
    }
}

fn escape_for_js_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

/// IPC callback invoked by WKWebView when JS calls `window.helios.postMessage(str)`.
/// For now, log the message. Full routing to Pi agent is wired via workspace event bus.
#[cfg(target_os = "macos")]
extern "C" fn canvas_ipc_callback(_context: *mut std::ffi::c_void, message: *const c_char) {
    if message.is_null() {
        return;
    }
    let c_str = unsafe { CStr::from_ptr(message) };
    if let Ok(msg) = c_str.to_str() {
        log::info!("[canvas-ipc] Received from webview: {}", msg);
        // TODO: Parse msg as JSON, extract type field, and dispatch:
        // - "interview_response" → route to Pi agent session
        // - "design_deck_response" → route to Pi agent session
        // For now the message is logged; full routing requires access to the
        // CLIAgentSessionsModel which isn't available in a static callback.
        // The workspace subscriber can poll for responses via a shared queue.
    }
}

impl Entity for CanvasView {
    type Event = PaneEvent;
}

impl View for CanvasView {
    fn render(&self, app: &AppContext) -> Box<dyn Element> {
        if self.palette_open {
            return self.render_palette(app);
        }

        if self.tabs.is_empty() {
            return Container::new(
                warpui::elements::Align::new(self.render_empty_state(app)).finish(),
            )
            .finish();
        }

        Flex::column()
            .with_children([
                self.render_tab_bar(app),
                Container::new(
                    Expanded::new(1.0, Empty::new().finish()).finish(),
                )
                .finish(),
            ])
            .with_main_axis_size(MainAxisSize::Max)
            .finish()
    }

    fn ui_name() -> &'static str {
        "CanvasView"
    }
}

impl TypedActionView for CanvasView {
    type Action = CanvasAction;

    fn handle_action(&mut self, action: &Self::Action, ctx: &mut ViewContext<Self>) {
        match action {
            CanvasAction::OpenTab {
                view_key,
                label,
                metadata,
            } => {
                self.open_tab(view_key.clone(), label.clone(), metadata.clone(), ctx);
            }
            CanvasAction::CloseTab { tab_id } => {
                self.close_tab(tab_id, ctx);
            }
            CanvasAction::ActivateTab { tab_id } => {
                self.activate_tab(tab_id, ctx);
            }
            CanvasAction::PinTab { tab_id } => {
                self.toggle_pin(tab_id, ctx);
            }
            CanvasAction::TogglePalette => {
                self.palette_open = !self.palette_open;
                ctx.notify();
            }
            CanvasAction::PaletteSelect { view_key } => {
                self.palette_open = false;
                let label = resolve_canvas_view(view_key)
                    .map(|r| r.label)
                    .unwrap_or_else(|| view_key.to_string());
                self.open_tab(view_key.clone(), label, HashMap::new(), ctx);
            }
        }
    }
}

impl BackingView for CanvasView {
    type PaneHeaderOverflowMenuAction = ();
    type CustomAction = ();
    type AssociatedData = ();

    fn handle_pane_header_overflow_menu_action(
        &mut self,
        _action: &Self::PaneHeaderOverflowMenuAction,
        _ctx: &mut ViewContext<Self>,
    ) {
    }

    fn close(&mut self, ctx: &mut ViewContext<Self>) {
        #[cfg(target_os = "macos")]
        {
            self.webviews.borrow_mut().clear();
        }
        ctx.emit(PaneEvent::Close);
    }

    fn focus_contents(&mut self, _ctx: &mut ViewContext<Self>) {}

    fn render_header_content(
        &self,
        _ctx: &view::HeaderRenderContext<'_>,
        _app: &AppContext,
    ) -> view::HeaderContent {
        let title = self
            .active_tab_id
            .as_ref()
            .and_then(|id| self.tabs.iter().find(|t| &t.id == id))
            .map(|t| t.label.as_str())
            .unwrap_or("Canvas");
        view::HeaderContent::simple(title)
    }

    fn set_focus_handle(&mut self, focus_handle: PaneFocusHandle, _ctx: &mut ViewContext<Self>) {
        self.focus_handle = Some(focus_handle);
    }
}
