use std::cell::RefCell;

use warpui::{AppContext, ModelHandle, View, ViewContext, ViewHandle};

use crate::app_state::{LeafContents, WebViewPaneSnapshot};
use crate::pane_group::pane::webview_view::WebViewView;

use super::{
    view::PaneView, BackingView, DetachType, PaneConfiguration, PaneContent, PaneGroup, PaneId, ShareableLink,
    ShareableLinkError,
};

pub struct WebViewPane {
    view: ViewHandle<PaneView<WebViewView>>,
    pane_configuration: ModelHandle<PaneConfiguration>,
    url: String,
    title: String,
    #[cfg(target_os = "macos")]
    native_webview: RefCell<Option<warpui::platform::mac::webview::HeliosWebView>>,
}

impl WebViewPane {
    pub fn new<V: View>(url: String, title: String, ctx: &mut ViewContext<V>) -> Self {
        let webview_view = ctx.add_typed_action_view(|ctx| WebViewView::new(title.clone(), url.clone(), ctx));
        let pane_configuration = webview_view.as_ref(ctx).pane_configuration();
        let pane_view = ctx.add_typed_action_view(|ctx| {
            let pane_id = PaneId::from_webview_pane_ctx(ctx);
            PaneView::new(
                pane_id,
                webview_view,
                (),
                pane_configuration.clone(),
                ctx,
            )
        });
        Self {
            view: pane_view,
            pane_configuration,
            url,
            title,
            #[cfg(target_os = "macos")]
            native_webview: RefCell::new(None),
        }
    }
}

impl PaneContent for WebViewPane {
    fn id(&self) -> PaneId {
        PaneId::from_webview_pane_view(&self.view)
    }

    fn attach(
        &self,
        _group: &PaneGroup,
        focus_handle: crate::pane_group::focus_state::PaneFocusHandle,
        ctx: &mut ViewContext<PaneGroup>,
    ) {
        self.view
            .update(ctx, |view, ctx| view.set_focus_handle(focus_handle, ctx));
        let child = self.view.as_ref(ctx).child(ctx);
        let pane_id = self.id();
        ctx.subscribe_to_view(&child, move |pane_group, _, event, ctx| {
            pane_group.handle_pane_event(pane_id, event, ctx);
        });

        // Create native WKWebView and add to window content view
        #[cfg(target_os = "macos")]
        {
            use cocoa::foundation::{NSPoint, NSRect, NSSize};
            use warpui::platform::mac::webview::HeliosWebView;
            use warpui::platform::mac::content_view_from_platform_window;

            let window_id = ctx.window_id();
            if let Some(platform_window) = ctx.windows().platform_window(window_id) {
                if let Some(content_view) = content_view_from_platform_window(platform_window.as_ref()) {
                    // Start with a reasonable default frame — will be updated on layout
                    let frame = NSRect::new(
                        NSPoint::new(0.0, 0.0),
                        NSSize::new(800.0, 600.0),
                    );
                    let webview = HeliosWebView::new(frame, Some(&self.url));
                    webview.add_to_view(content_view);
                    *self.native_webview.borrow_mut() = Some(webview);
                }
            }
        }
    }

    fn detach(
        &self,
        _group: &PaneGroup,
        _detach_type: DetachType,
        ctx: &mut ViewContext<PaneGroup>,
    ) {
        let child = self.view.as_ref(ctx).child(ctx);
        ctx.unsubscribe_to_view(&child);

        // Remove native webview (Drop calls helios_webview_release)
        #[cfg(target_os = "macos")]
        {
            *self.native_webview.borrow_mut() = None;
        }
    }

    fn snapshot(&self, _ctx: &AppContext) -> LeafContents {
        LeafContents::WebView(WebViewPaneSnapshot {
            url: self.url.clone(),
            title: self.title.clone(),
        })
    }

    fn has_application_focus(&self, ctx: &mut ViewContext<PaneGroup>) -> bool {
        self.view.is_self_or_child_focused(ctx)
    }

    fn focus(&self, ctx: &mut ViewContext<PaneGroup>) {
        self.view.as_ref(ctx).child(ctx)
            .update(ctx, |view, ctx| view.focus_contents(ctx));
    }

    fn shareable_link(
        &self,
        _ctx: &mut ViewContext<PaneGroup>,
    ) -> Result<ShareableLink, ShareableLinkError> {
        Ok(ShareableLink::Base)
    }

    fn pane_configuration(&self) -> ModelHandle<PaneConfiguration> {
        self.pane_configuration.clone()
    }

    fn is_pane_being_dragged(&self, ctx: &AppContext) -> bool {
        self.view.as_ref(ctx).is_being_dragged()
    }
}
