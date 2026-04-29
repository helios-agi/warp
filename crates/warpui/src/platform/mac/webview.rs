use cocoa::base::id;
use cocoa::foundation::NSRect;
use std::ffi::CString;
use std::os::raw::c_char;

// FFI declarations matching webview.h
extern "C" {
    fn helios_webview_create(
        frame: NSRect,
        initial_url: *const c_char,
        callback: Option<extern "C" fn(*mut std::ffi::c_void, *const c_char)>,
        context: *mut std::ffi::c_void,
    ) -> id;
    fn helios_webview_load_url(webview: id, url: *const c_char);
    fn helios_webview_load_html(webview: id, html: *const c_char);
    fn helios_webview_set_frame(webview: id, frame: NSRect);
    fn helios_webview_add_to_view(webview: id, parent_view: id);
    fn helios_webview_remove(webview: id);
    fn helios_webview_eval_js(webview: id, js: *const c_char);
    fn helios_webview_release(webview: id);
    fn helios_webview_set_autoresize(webview: id);
}

/// Safe wrapper around WKWebView
pub struct HeliosWebView {
    native: id,
}

impl HeliosWebView {
    pub fn new(
        frame: NSRect,
        url: Option<&str>,
        callback: Option<extern "C" fn(*mut std::ffi::c_void, *const c_char)>,
        context: *mut std::ffi::c_void,
    ) -> Self {
        let c_url = url.and_then(|u| CString::new(u).ok());
        let ptr = c_url.as_ref().map(|c| c.as_ptr()).unwrap_or(std::ptr::null());
        let native = unsafe { helios_webview_create(frame, ptr, callback, context) };
        Self { native }
    }

    pub fn load_url(&self, url: &str) {
        let Ok(c_url) = CString::new(url) else { return }; // URL contains NUL byte, skip
        unsafe { helios_webview_load_url(self.native, c_url.as_ptr()) };
    }

    pub fn load_html(&self, html: &str) {
        let Ok(c_html) = CString::new(html) else { return }; // HTML contains NUL byte, skip
        unsafe { helios_webview_load_html(self.native, c_html.as_ptr()) };
    }

    pub fn set_frame(&self, frame: NSRect) {
        unsafe { helios_webview_set_frame(self.native, frame) };
    }

    pub fn add_to_view(&self, parent: id) {
        unsafe { helios_webview_add_to_view(self.native, parent) };
    }

    pub fn remove(&self) {
        unsafe { helios_webview_remove(self.native) };
    }

    pub fn eval_js(&self, js: &str) {
        let Ok(c_js) = CString::new(js) else { return }; // JS contains NUL byte, skip
        unsafe { helios_webview_eval_js(self.native, c_js.as_ptr()) };
    }

    pub fn native_id(&self) -> id {
        self.native
    }

    pub fn set_autoresize(&self) {
        unsafe { helios_webview_set_autoresize(self.native) };
    }
}

impl Drop for HeliosWebView {
    fn drop(&mut self) {
        unsafe { helios_webview_release(self.native) };
    }
}
