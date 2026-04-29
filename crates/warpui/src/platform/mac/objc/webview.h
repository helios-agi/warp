#import <AppKit/AppKit.h>
#import <WebKit/WebKit.h>

// Per-instance IPC callback with context pointer
typedef void (*helios_webview_ipc_callback)(void* context, const char* message);

// Create a WKWebView with per-instance IPC callback
id helios_webview_create(NSRect frame, const char* initial_url,
                         helios_webview_ipc_callback callback, void* context);

// Load a URL
void helios_webview_load_url(id webview, const char* url);

// Load HTML string
void helios_webview_load_html(id webview, const char* html);

// Set frame (reposition/resize)
void helios_webview_set_frame(id webview, NSRect frame);

// Add as subview of a parent NSView
void helios_webview_add_to_view(id webview, id parent_view);

// Remove from parent
void helios_webview_remove(id webview);

// Execute JavaScript
void helios_webview_eval_js(id webview, const char* js);


// Release webview and break retain cycles
void helios_webview_release(id webview);
