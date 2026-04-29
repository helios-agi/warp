#import "webview.h"

// Global IPC callback pointer (set from Rust)
static helios_webview_ipc_callback _ipc_callback = NULL;

// Message handler delegate
@interface HeliosMessageHandler : NSObject <WKScriptMessageHandler>
@end

@implementation HeliosMessageHandler
- (void)userContentController:(WKUserContentController *)controller
      didReceiveScriptMessage:(WKScriptMessage *)message {
    if (_ipc_callback && [message.body isKindOfClass:[NSString class]]) {
        const char* body = [(NSString*)message.body UTF8String];
        _ipc_callback(body);
    }
}
@end

static HeliosMessageHandler* _handler = nil;

id helios_webview_create(NSRect frame, const char* initial_url) {
    WKWebViewConfiguration* config = [[WKWebViewConfiguration alloc] init];

    // Set up IPC message handler
    _handler = [[HeliosMessageHandler alloc] init];
    [config.userContentController addScriptMessageHandler:_handler name:@"helios"];

    // Inject IPC bridge script
    NSString* bridge = @"window.helios = { postMessage: function(msg) { "
                        "window.webkit.messageHandlers.helios.postMessage("
                        "typeof msg === 'string' ? msg : JSON.stringify(msg)); } };";
    WKUserScript* script = [[WKUserScript alloc]
        initWithSource:bridge
        injectionTime:WKUserScriptInjectionTimeAtDocumentStart
        forMainFrameOnly:YES];
    [config.userContentController addUserScript:script];

    // Enable developer tools in debug builds
#ifdef DEBUG
    [config.preferences setValue:@YES forKey:@"developerExtrasEnabled"];
#endif

    WKWebView* webview = [[WKWebView alloc] initWithFrame:frame configuration:config];

    // Load initial URL if provided
    if (initial_url) {
        NSString* urlStr = [NSString stringWithUTF8String:initial_url];
        if ([urlStr hasPrefix:@"file://"]) {
            NSURL* url = [NSURL URLWithString:urlStr];
            NSURL* dir = [url URLByDeletingLastPathComponent];
            [webview loadFileURL:url allowingReadAccessToDirectory:dir];
        } else {
            NSURL* url = [NSURL URLWithString:urlStr];
            NSURLRequest* req = [NSURLRequest requestWithURL:url];
            [webview loadRequest:req];
        }
    }

    return webview;
}

void helios_webview_load_url(id webview, const char* url) {
    NSString* urlStr = [NSString stringWithUTF8String:url];
    NSURL* nsurl = [NSURL URLWithString:urlStr];
    if ([urlStr hasPrefix:@"file://"]) {
        NSURL* dir = [nsurl URLByDeletingLastPathComponent];
        [(WKWebView*)webview loadFileURL:nsurl allowingReadAccessToDirectory:dir];
    } else {
        NSURLRequest* req = [NSURLRequest requestWithURL:nsurl];
        [(WKWebView*)webview loadRequest:req];
    }
}

void helios_webview_load_html(id webview, const char* html) {
    NSString* htmlStr = [NSString stringWithUTF8String:html];
    [(WKWebView*)webview loadHTMLString:htmlStr baseURL:nil];
}

void helios_webview_set_frame(id webview, NSRect frame) {
    [(WKWebView*)webview setFrame:frame];
}

void helios_webview_add_to_view(id webview, id parent_view) {
    [(NSView*)parent_view addSubview:(WKWebView*)webview];
}

void helios_webview_remove(id webview) {
    [(WKWebView*)webview removeFromSuperview];
}

void helios_webview_eval_js(id webview, const char* js) {
    NSString* jsStr = [NSString stringWithUTF8String:js];
    [(WKWebView*)webview evaluateJavaScript:jsStr completionHandler:nil];
}

void helios_webview_set_ipc_callback(helios_webview_ipc_callback callback) {
    _ipc_callback = callback;
}
