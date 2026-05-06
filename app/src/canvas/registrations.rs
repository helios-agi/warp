use super::registry::{register_canvas_view, CanvasViewRegistration};
use super::types::{CanvasViewCategory, CanvasViewKind};

pub fn register_builtin_views() {
    register_canvas_view(CanvasViewRegistration {
        key: "interview".into(),
        label: "Interview".into(),
        category: CanvasViewCategory::Core,
        kind: CanvasViewKind::WebView {
            filename: "interview.html".into(),
        },
        icon: Some("chat_circle".into()),
    });

    register_canvas_view(CanvasViewRegistration {
        key: "design-deck".into(),
        label: "Design Deck".into(),
        category: CanvasViewCategory::Tools,
        kind: CanvasViewKind::WebView {
            filename: "design-deck.html".into(),
        },
        icon: Some("stack_simple".into()),
    });

    register_canvas_view(CanvasViewRegistration {
        key: "inbox".into(),
        label: "Inbox".into(),
        category: CanvasViewCategory::Core,
        kind: CanvasViewKind::WebView {
            filename: "inbox.html".into(),
        },
        icon: Some("envelope".into()),
    });

    register_canvas_view(CanvasViewRegistration {
        key: "crm".into(),
        label: "CRM".into(),
        category: CanvasViewCategory::Tools,
        kind: CanvasViewKind::WebView {
            filename: "crm.html".into(),
        },
        icon: Some("users".into()),
    });

    register_canvas_view(CanvasViewRegistration {
        key: "helios:brain".into(),
        label: "Brain".into(),
        category: CanvasViewCategory::Intelligence,
        kind: CanvasViewKind::WebView {
            filename: "brain.html".into(),
        },
        icon: Some("brain".into()),
    });

    register_canvas_view(CanvasViewRegistration {
        key: "helios:mesh".into(),
        label: "Mesh".into(),
        category: CanvasViewCategory::Intelligence,
        kind: CanvasViewKind::WebView {
            filename: "mesh.html".into(),
        },
        icon: Some("graph".into()),
    });

    register_canvas_view(CanvasViewRegistration {
        key: "helios:warmloop".into(),
        label: "Warm Loop".into(),
        category: CanvasViewCategory::Intelligence,
        kind: CanvasViewKind::WebView {
            filename: "warmloop.html".into(),
        },
        icon: Some("arrows_clockwise".into()),
    });

    register_canvas_view(CanvasViewRegistration {
        key: "helios:cortex".into(),
        label: "Cortex".into(),
        category: CanvasViewCategory::Intelligence,
        kind: CanvasViewKind::WebView {
            filename: "cortex.html".into(),
        },
        icon: Some("lightning".into()),
    });

    register_canvas_view(CanvasViewRegistration {
        key: "helios:governance".into(),
        label: "Governance".into(),
        category: CanvasViewCategory::Admin,
        kind: CanvasViewKind::WebView {
            filename: "governance.html".into(),
        },
        icon: Some("shield".into()),
    });

    register_canvas_view(CanvasViewRegistration {
        key: "helios:pulse".into(),
        label: "Pulse".into(),
        category: CanvasViewCategory::Intelligence,
        kind: CanvasViewKind::WebView {
            filename: "pulse.html".into(),
        },
        icon: Some("pulse".into()),
    });
}
