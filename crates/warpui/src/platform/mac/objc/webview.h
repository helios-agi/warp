#import <AppKit/AppKit.h>
#import <WebKit/WebKit.h>

// Per-instance IPC callback with context pointer
typedef void (*warp_webview_ipc_callback)(void* context, const char* message);

// Create a WKWebView with per-instance IPC callback
id warp_webview_create(NSRect frame, const char* initial_url,
                       warp_webview_ipc_callback callback, void* context);

// Load a URL
void warp_webview_load_url(id webview, const char* url);

// Load HTML string
void warp_webview_load_html(id webview, const char* html);

// Set frame (reposition/resize)
void warp_webview_set_frame(id webview, NSRect frame);

// Add as subview of a parent NSView
void warp_webview_add_to_view(id webview, id parent_view);

// Remove from parent
void warp_webview_remove(id webview);

// Execute JavaScript
void warp_webview_eval_js(id webview, const char* js);


// Release webview and break retain cycles
void warp_webview_release(id webview);

// Set autoresizing mask so WKWebView fills its parent on resize
void warp_webview_set_autoresize(id webview);
