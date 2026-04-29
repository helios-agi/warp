#import "webview.h"

// Shared process pool for all webviews (memory/process efficiency)
static WKProcessPool* _sharedProcessPool = nil;

static WKProcessPool* sharedProcessPool(void) {
    if (!_sharedProcessPool) {
        _sharedProcessPool = [[WKProcessPool alloc] init];
    }
    return _sharedProcessPool;
}

// Message handler delegate — stores per-instance callback and context pointer
@interface WarpWebViewMessageHandler : NSObject <WKScriptMessageHandler> {
    warp_webview_ipc_callback _callback;
    void* _context;
}
- (instancetype)initWithCallback:(warp_webview_ipc_callback)callback context:(void*)context;
@end

@implementation WarpWebViewMessageHandler
- (instancetype)initWithCallback:(warp_webview_ipc_callback)callback context:(void*)context {
    self = [super init];
    if (self) {
        _callback = callback;
        _context = context;
    }
    return self;
}

- (void)userContentController:(WKUserContentController *)controller
      didReceiveScriptMessage:(WKScriptMessage *)message {
    if (_callback && [message.body isKindOfClass:[NSString class]]) {
        const char* body = [(NSString*)message.body UTF8String];
        _callback(_context, body);
    }
}
@end

id warp_webview_create(NSRect frame, const char* initial_url,
                       warp_webview_ipc_callback callback, void* context) {
    WKWebViewConfiguration* config = [[WKWebViewConfiguration alloc] init];

    // Use shared process pool for memory efficiency
    config.processPool = sharedProcessPool();

    // Set up per-instance IPC message handler
    WarpWebViewMessageHandler* handler = [[WarpWebViewMessageHandler alloc] initWithCallback:callback context:context];
    [config.userContentController addScriptMessageHandler:handler name:@"helios"];

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
            if (!url) {
                // Try percent-encoding for malformed URLs
                NSString* encoded = [urlStr stringByAddingPercentEncodingWithAllowedCharacters:[NSCharacterSet URLQueryAllowedCharacterSet]];
                url = [NSURL URLWithString:encoded];
            }
            if (url) {
                // Restrict read access to the webviews/ bundle directory only
                NSString* webviewsDir = [[[NSBundle mainBundle] resourceURL]
                    URLByAppendingPathComponent:@"webviews"].path;
                NSURL* allowedDir;
                if (webviewsDir && [[NSFileManager defaultManager] fileExistsAtPath:webviewsDir]) {
                    allowedDir = [NSURL fileURLWithPath:webviewsDir];
                } else {
                    // Fallback to file's own directory (dev builds)
                    allowedDir = [url URLByDeletingLastPathComponent];
                }
                [webview loadFileURL:url allowingReadAccessToDirectory:allowedDir];
            }
        } else {
            NSURL* url = [NSURL URLWithString:urlStr];
            if (!url) {
                // Try percent-encoding for malformed URLs
                NSString* encoded = [urlStr stringByAddingPercentEncodingWithAllowedCharacters:[NSCharacterSet URLQueryAllowedCharacterSet]];
                url = [NSURL URLWithString:encoded];
            }
            if (url) {
                NSURLRequest* req = [NSURLRequest requestWithURL:url];
                [webview loadRequest:req];
            }
        }
    }

    return webview;
}

void warp_webview_load_url(id webview, const char* url) {
    NSString* urlStr = [NSString stringWithUTF8String:url];
    NSURL* nsurl = [NSURL URLWithString:urlStr];
    if (!nsurl) {
        // Try percent-encoding for malformed URLs
        NSString* encoded = [urlStr stringByAddingPercentEncodingWithAllowedCharacters:[NSCharacterSet URLQueryAllowedCharacterSet]];
        nsurl = [NSURL URLWithString:encoded];
    }
    if (!nsurl) return; // Invalid URL, skip
    
    if ([urlStr hasPrefix:@"file://"]) {
        // Restrict read access to the webviews/ bundle directory only
        NSString* webviewsDir = [[[NSBundle mainBundle] resourceURL]
            URLByAppendingPathComponent:@"webviews"].path;
        NSURL* allowedDir;
        if (webviewsDir && [[NSFileManager defaultManager] fileExistsAtPath:webviewsDir]) {
            allowedDir = [NSURL fileURLWithPath:webviewsDir];
        } else {
            // Fallback to file's own directory (dev builds)
            allowedDir = [nsurl URLByDeletingLastPathComponent];
        }
        [(WKWebView*)webview loadFileURL:nsurl allowingReadAccessToDirectory:allowedDir];
    } else {
        NSURLRequest* req = [NSURLRequest requestWithURL:nsurl];
        [(WKWebView*)webview loadRequest:req];
    }
}

void warp_webview_load_html(id webview, const char* html) {
    NSString* htmlStr = [NSString stringWithUTF8String:html];
    [(WKWebView*)webview loadHTMLString:htmlStr baseURL:nil];
}

void warp_webview_set_frame(id webview, NSRect frame) {
    [(WKWebView*)webview setFrame:frame];
}

void warp_webview_add_to_view(id webview, id parent_view) {
    [(NSView*)parent_view addSubview:(WKWebView*)webview];
}

void warp_webview_remove(id webview) {
    [(WKWebView*)webview removeFromSuperview];
}

void warp_webview_eval_js(id webview, const char* js) {
    NSString* jsStr = [NSString stringWithUTF8String:js];
    [(WKWebView*)webview evaluateJavaScript:jsStr completionHandler:nil];
}

void warp_webview_release(id webview) {
    WKWebView* wv = (WKWebView*)webview;
    // Remove message handler to break retain cycle
    [wv.configuration.userContentController removeScriptMessageHandlerForName:@"helios"];
    [wv removeFromSuperview];
    // Under MRC, this balances the +1 from alloc/init
    #if !__has_feature(objc_arc)
    [wv release];
    #endif
}

void warp_webview_set_autoresize(id webview) {
    WKWebView* wv = (WKWebView*)webview;
    wv.autoresizingMask = NSViewWidthSizable | NSViewHeightSizable;
}
