mod app;
mod components;
mod win;

use app::CarteroApplication;
use gtk4::prelude::*;
pub use win::CarteroWindow;

fn main() -> glib::ExitCode {
    let app = CarteroApplication::new();
    app.run()
}
