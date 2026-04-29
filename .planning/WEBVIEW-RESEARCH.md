# WebView Pane — Implementation Research & Sources

## Helios Terminal Stack Context

### Current ObjC FFI Pattern (Warp)
- **Crate**: `objc` (not `objc2`) + `cocoa` for AppKit bindings
- **ObjC compilation**: `cc::Build` compiles `.m` files into `warp_objc` static library
- **Framework linking**: via `cargo:rustc-link-lib=framework=X` in build.rs
- **Message passing**: `msg_send![]` macro from `objc` crate
- **Window hierarchy**: `NSWindow` → `WarpHostView` (custom `NSView` subclass with Metal `CALayer`)
- **Content view access**: `msg_send![native_window, contentView]` returns the `WarpHostView`

### Key Files
```
crates/warpui/build.rs              — Compiles ObjC, links frameworks
crates/warpui/src/platform/mac/
  objc/host_view.h/.m               — WarpHostView (NSView + Metal CALayer)
  objc/menus.h/.m                   — Native macOS menus
  objc/app.h/.m                     — NSApplication delegate
  window.rs                         — Rust ↔ NSWindow bridge
  menus.rs                          — Rust ↔ NSMenu bridge
```

---

## Source 1: Tauri/wry — WKWebView Implementation (REFERENCE IMPLEMENTATION)
**URL**: https://github.com/tauri-apps/wry/blob/dev/src/wkwebview/mod.rs
**Relevance**: 🔴 Critical — production-grade WKWebView in Rust, ~1500 lines
**Key code**:
```rust
// wry creates WKWebView and adds it as subview of NSView
let webview: Retained<WryWebView> =
    objc2::msg_send![super(webview), initWithFrame: frame, configuration: &**config];

// For child webviews (our use case):
if is_child {
    ns_view.addSubview(&webview);
} else {
    // Full window replacement
    ns_window.setContentView(Some(&parent_view));
}
```
**IPC bridge**:
```rust
// JavaScript → Rust: WKScriptMessageHandler
Object.defineProperty(window, 'ipc', {
  value: Object.freeze({
    postMessage: function(s) {
      window.webkit.messageHandlers.ipc.postMessage(s);
    }
  })
});

// Rust → JavaScript: evaluateJavaScript
self.webview.evaluateJavaScript_completionHandler(&NSString::from_str(js), Some(&handler));
```

**Note**: wry uses `objc2` crate (newer). Warp uses `objc` (older). Need to use `msg_send!` from `objc` crate instead.

---

## Source 2: cacao — Rust AppKit WebView Bindings
**URL**: https://github.com/ryanmcgrath/cacao/blob/trunk/src/webview/mod.rs
**Docs**: https://docs.rs/cacao/latest/cacao/webview/index.html
**Relevance**: 🟡 High — simpler API, shows the pattern without objc2 complexity
**Key pattern**:
```rust
// cacao wraps WKWebView with safe Rust API
pub struct WebView<T = ()> {
    pub objc: ObjcProperty,
    // ...
}

impl WebView {
    pub fn new(config: WebViewConfig) -> Self { ... }
    pub fn load_url(&self, url: &str) { ... }
    pub fn load_html_string(&self, html: &str, base_url: &str) { ... }
}
```

---

## Source 3: objc2-web-kit — Typed Rust Bindings for WebKit
**URL**: https://docs.rs/objc2-web-kit/latest/objc2_web_kit/struct.WKWebView.html
**Relevance**: 🟡 Reference — shows all WKWebView methods available
**Key API**:
- `initWithFrame_configuration(frame, config)` — Create WKWebView
- `loadRequest(request)` — Load URL
- `loadHTMLString_baseURL(html, url)` — Load HTML string
- `evaluateJavaScript_completionHandler(js, handler)` — Execute JS
- `setFrame(rect)` — Position/resize

---

## Source 4: Apple — WKWebView Documentation
**URL**: https://developer.apple.com/documentation/webkit/wkwebview
**Relevance**: 🟡 Reference — canonical API docs
**Key points**:
- WKWebView is an NSView subclass (can be added as subview)
- Requires WebKit.framework
- Configuration via WKWebViewConfiguration
- IPC via WKUserContentController + WKScriptMessageHandler

---

## Source 5: Apple — NSView addSubview
**URL**: https://developer.apple.com/documentation/appkit/nsview/addsubview(_:)
**Relevance**: 🟢 Essential — how to add WebView to Warp's view hierarchy
**Pattern**: `[parentView addSubview:webView]` / `msg_send![parent_view, addSubview: web_view]`

---

## Source 6: Apple — addSubview:positioned:relativeTo:
**URL**: https://developer.apple.com/documentation/appkit/nsview/addsubview(_:positioned:relativeto:)
**Relevance**: 🟢 Essential — control z-ordering when mixing Metal + WKWebView
**Pattern**: Can position WebView above or below Metal CALayer content

---

## Source 7: Apple — WKScriptMessageHandler
**URL**: https://developer.apple.com/documentation/webkit/wkscriptmessagehandler
**Relevance**: 🟢 Essential — JavaScript → Rust IPC
**Pattern**:
```objc
// JS side:
window.webkit.messageHandlers.helios.postMessage({type: "query", cypher: "..."});

// ObjC side:
- (void)userContentController:(WKUserContentController *)userContentController
      didReceiveScriptMessage:(WKScriptMessage *)message {
    // Forward to Rust via C function pointer
}
```

---

## Source 8: wry IPC Deep Dive
**URL**: https://deepwiki.com/tauri-apps/wry/4.2-ipc-(inter-process-communication)
**Relevance**: 🟡 Architecture reference
**Key insight**: wry registers a custom URL scheme handler for IPC, not just postMessage. This allows binary data transfer and async responses.

---

## Source 9: darwin_webkit — Older Rust WebKit Bindings
**URL**: https://yamadapc.github.io/rust-darwin-webkit/darwin_webkit/
**Relevance**: 🟢 Shows `objc` crate (v1) patterns matching Warp's stack
**Key code** (uses same `objc` crate as Warp):
```rust
use objc::msg_send;
use cocoa::base::id;

unsafe {
    let config: id = msg_send![class!(WKWebViewConfiguration), new];
    let web_view: id = msg_send![class!(WKWebView), alloc];
    let web_view: id = msg_send![web_view, initWithFrame:frame configuration:config];
}
```

---

## Source 10: Warp's Window/View Bridge (INTERNAL)
**File**: `crates/warpui/src/platform/mac/window.rs`
**Relevance**: 🔴 Critical — shows how Warp accesses NSWindow/NSView
**Key code**:
```rust
// Getting content view (WarpHostView):
let native_view: id = msg_send![native_window, contentView];

// Frame access:
let view_frame = unsafe { NSView::frame(self.native_window.contentView()) };

// Scale factor for HiDPI:
unsafe { NSWindow::backingScaleFactor(self.native_window) }
```

---

## Source 11: Warp's ObjC Build System (INTERNAL)
**File**: `crates/warpui/build.rs`
**Relevance**: 🔴 Critical — shows exactly how to add new ObjC files
**Pattern**:
```rust
// In compile_objc_lib():
println!("cargo:rustc-link-lib=framework=WebKit");  // ADD THIS
println!("cargo:rerun-if-changed=src/platform/mac/objc/webview.m");  // ADD THIS

cc::Build::new()
    .file("src/platform/mac/objc/webview.m")  // ADD THIS
    // ... existing files ...
    .compile("warp_objc");
```

---

## Source 12: Warp's WarpHostView (INTERNAL)
**File**: `crates/warpui/src/platform/mac/objc/host_view.h/.m`
**Relevance**: 🔴 Critical — the parent view where WebView will be added
**Key insight**: WarpHostView is the content view of the window. It's backed by a Metal CALayer. Adding a WKWebView as a subview will overlay it on top of the Metal-rendered content.

---

## Source 13: Warp's Element System (INTERNAL)
**File**: `crates/warpui/src/elements/`
**Relevance**: 🟡 Architecture — how Warp positions UI elements
**Key insight**: Warp's element tree calculates layout in Rust, then renders via Metal. A WebView overlay needs to track position/size from the element tree and apply it to the native NSView frame.

---

## Source 14: cocoa crate — NSView trait
**URL**: https://yamadapc.github.io/rust-darwin-webkit/cocoa/appkit/trait.NSView.html
**Relevance**: 🟢 API reference for the `cocoa` crate Warp uses
**Key methods**:
- `initWithFrame_(rect)` — Create view with frame
- `frame()` → `NSRect` — Get view frame
- `setFrame_(rect)` — Set view frame
- `addSubview_(view)` — Add child view
- `removeFromSuperview()` — Remove from parent

---

## Source 15: Blog — Building macOS Apps in Rust
**URL**: https://rymc.io/blog/2021/cacao-rs-macos-ios-rust/
**Relevance**: 🟢 Patterns — general macOS Rust architecture
**Key insight**: The author of cacao discusses view hierarchy, delegate patterns, and memory management with NSView in Rust.

---

## Source 16: Blog — How to Create a macOS Cocoa App in Rust
**URL**: https://blog.seanvoss.com/how-to-create-a-macos-cocoa-app-in-rust-part-1/
**Relevance**: 🟢 Tutorial — step-by-step NSView creation with `cocoa` crate

---

## Source 17: Medium — Developing macOS Applications in Rust
**URL**: https://medium.com/@alfred.weirich/developing-macos-applications-in-rust-7baefc9894db
**Relevance**: 🟢 Patterns — covers window creation, view hierarchy

---

## Source 18: Apple — WKWebViewConfiguration
**URL**: https://developer.apple.com/documentation/webkit/wkwebviewconfiguration
**Relevance**: 🟢 API — configuration for custom protocols, cookies, preferences

---

## Source 19: Apple — WKUserContentController
**URL**: https://developer.apple.com/documentation/webkit/wkusercontentcontroller
**Relevance**: 🟢 API — manages JavaScript injection and message handlers

---

## Source 20: Apple — addScriptMessageHandler
**URL**: https://developer.apple.com/documentation/webkit/wkusercontentcontroller/addscriptmessagehandler(_:contentworld:name:)
**Relevance**: 🟢 API — registers Rust callbacks for JS postMessage calls

---

## Source 21: Stack Overflow — Load NSView Dynamically
**URL**: https://stackoverflow.com/questions/4898229/how-to-load-an-nsview-on-an-nswindow-dynamically
**Relevance**: 🟢 Pattern — adding/removing subviews dynamically

---

## Source 22: Hacking with Swift — WKWebView Guide
**URL**: https://www.hackingwithswift.com/articles/112/the-ultimate-guide-to-wkwebview
**Relevance**: 🟡 Comprehensive reference — all WKWebView features explained
**Key topics**: Navigation, JavaScript evaluation, custom schemes, cookies, file loading

---

## Source 23: Medium — JS Bridge for WKWebView
**URL**: https://medium.com/@bahalek/setting-up-js-bridge-between-your-webpage-and-wkwebview-in-your-ios-app-4ec8ca8230f7
**Relevance**: 🟢 Tutorial — bidirectional IPC implementation

---

## Source 24: Warp's Dialog System (INTERNAL)
**File**: `app/src/ui_components/dialog.rs`
**Relevance**: 🟡 Shows how Warp positions UI overlays
**Key insight**: Dialogs use absolute positioning. WebView pane needs similar layout coordination.

---

## Source 25: Warp's Resizable Panel (INTERNAL)
**File**: `crates/warpui/src/elements/resizable.rs`
**Relevance**: 🟡 Shows how right panel sizing works
**Key insight**: The right panel uses `Resizable` element with drag handles. WebView frame needs to sync with this.

---

## Source 26: wry's WryWebViewParent (Tauri)
**URL**: https://github.com/tauri-apps/wry/blob/dev/src/wkwebview/class/wry_web_view_parent.rs
**Relevance**: 🟢 Pattern — custom parent NSView that manages WebView positioning

---

## Source 27: Warp's PaneContent Trait (INTERNAL)
**File**: `app/src/pane_group/pane/mod.rs`
**Relevance**: 🔴 Critical — trait that all pane types implement
**Key insight**: New WebView pane must implement `PaneContent` to integrate with tab system

---

## Source 28: Warp's LeafContents Enum (INTERNAL)
**File**: `app/src/app_state.rs`
**Relevance**: 🔴 Critical — add `WebView(WebViewPaneSnapshot)` variant here

---

## Source 29: Warp's RightPanelView (INTERNAL)
**File**: `app/src/workspace/view/right_panel.rs`
**Relevance**: 🔴 Critical — extend to support WebView content alongside CodeReview

---

## Source 30: GPUI — Zed's GPU-rendered UI Framework
**URL**: https://www.blog.brightcoding.dev/2026/02/23/gpui-component-build-stunning-rust-desktop-apps-with-gpu-power
**Relevance**: 🟢 Architecture comparison — Zed also uses GPU rendering + native views
**Key insight**: Zed's GPUI framework faces the same challenge (GPU-rendered UI + native NSView overlays). They solve it by tracking element positions in the GPU layout tree and syncing native view frames.

---

## Source 31: Helios-Desktop Graph Driver (INTERNAL)
**File**: `/Users/chikochingaya/familiar/apps/helios-desktop/src/main/db/graph-driver.ts`
**Relevance**: 🟢 Bolt client — reusable for WebView ↔ Memgraph communication
**Pattern**: Shared neo4j-driver singleton with circuit breaker

---

## Source 32: Pi's Safe-Memgraph Client (INTERNAL)
**File**: `/Users/chikochingaya/.pi/agent/lib/safe-memgraph.js`
**Relevance**: 🟢 Bolt client — alternative for WebView ↔ Memgraph
**Pattern**: Broker-first with direct fallback

---

## Source 33: Helios-Desktop Canvas Tab Store (INTERNAL)
**File**: `/Users/chikochingaya/familiar/apps/helios-desktop/src/renderer/stores/canvas-tabs-store.ts`
**Relevance**: 🟡 Tab system architecture — reference for WebView tab management
**Key features**: Ghost tabs, preview tabs, split view, dirty state, max 20 tabs

---

## Source 34: Helios-Desktop View Registrations (INTERNAL)
**File**: `/Users/chikochingaya/familiar/apps/helios-desktop/src/renderer/components/canvas/view-registrations.ts`
**Relevance**: 🟡 Route registry — shows all 40+ views that WebView pane unlocks

---

## Summary: Implementation Path

### Phase 1: ObjC WebView Wrapper
1. Create `webview.h` / `webview.m` in `crates/warpui/src/platform/mac/objc/`
2. Pattern: Match existing `host_view.h/.m` style
3. Add `WebKit.framework` linking in `build.rs`
4. Functions: `helios_webview_create()`, `helios_webview_load_url()`, `helios_webview_set_frame()`, `helios_webview_remove()`

### Phase 2: Rust FFI Bridge
1. Create `webview.rs` in `crates/warpui/src/platform/mac/`
2. Pattern: Match `window.rs` FFI style using `msg_send!` from `objc` crate
3. Key: Track WebView `id` and sync frame with element layout

### Phase 3: Pane Integration
1. Add `LeafContents::WebView` to `app/src/app_state.rs`
2. Create `WebViewPane` in `app/src/pane_group/pane/webview_pane.rs`
3. Add `IPaneType::WebView` to enum
4. Wire into `RightPanelView` for interview/panel rendering

### Phase 4: IPC Bridge
1. JS side: `window.webkit.messageHandlers.helios.postMessage(JSON)`
2. ObjC side: `WKScriptMessageHandler` delegate forwards to Rust callback
3. Rust side: Parse JSON, route to Memgraph/Pi services
4. Response: `evaluateJavaScript` to push data back to WebView
