use warpui::{AppContext, ModelHandle, View, ViewContext, ViewHandle};

use crate::app_state::LeafContents;
use crate::canvas::canvas_view::CanvasView;

use super::view::PaneView;
use super::{
    BackingView, DetachType, PaneConfiguration, PaneContent, PaneId, ShareableLink,
    ShareableLinkError,
};
use crate::pane_group::PaneGroup;

pub struct CanvasPane {
    view: ViewHandle<PaneView<CanvasView>>,
    pane_configuration: ModelHandle<PaneConfiguration>,
}

impl CanvasPane {
    pub fn new<V: View>(ctx: &mut ViewContext<V>) -> Self {
        let canvas_view = ctx.add_typed_action_view(|ctx| CanvasView::new(ctx));
        let pane_configuration = canvas_view.as_ref(ctx).pane_configuration();
        let pane_view = ctx.add_typed_action_view(|ctx| {
            let pane_id = PaneId::from_canvas_pane_ctx(ctx);
            PaneView::new(pane_id, canvas_view, (), pane_configuration.clone(), ctx)
        });
        Self {
            view: pane_view,
            pane_configuration,
        }
    }

    #[allow(dead_code)]
    pub fn canvas_view_handle<'a>(&self, ctx: &'a AppContext) -> &'a PaneView<CanvasView> {
        self.view.as_ref(ctx)
    }
}

impl PaneContent for CanvasPane {
    fn id(&self) -> PaneId {
        PaneId::from_canvas_pane_view(&self.view)
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
    }

    fn detach(
        &self,
        _group: &PaneGroup,
        _detach_type: DetachType,
        ctx: &mut ViewContext<PaneGroup>,
    ) {
        let child = self.view.as_ref(ctx).child(ctx);
        ctx.unsubscribe_to_view(&child);
    }

    fn snapshot(&self, _ctx: &AppContext) -> LeafContents {
        LeafContents::Canvas
    }

    fn has_application_focus(&self, ctx: &mut ViewContext<PaneGroup>) -> bool {
        self.view.is_self_or_child_focused(ctx)
    }

    fn focus(&self, ctx: &mut ViewContext<PaneGroup>) {
        self.view
            .as_ref(ctx)
            .child(ctx)
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
