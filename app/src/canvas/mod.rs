pub mod canvas_view;
pub mod registrations;
pub mod registry;
pub mod types;

use warpui::AppContext;

pub fn init(_app: &mut AppContext) {
    registrations::register_builtin_views();
}
