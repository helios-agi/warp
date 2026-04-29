#import <AppKit/AppKit.h>
#import <WebKit/WebKit.h>

// Create a WKWebView with IPC message handler, return as id
id helios_webview_create(NSRect frame, const char* initial_url);

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

// Set the IPC callback (Rust function pointer)
typedef void (*helios_webview_ipc_callback)(const char* message);
void helios_webview_set_ipc_callback(helios_webview_ipc_callback callback);
