use glib::Object;
use gtk4::gio;

use crate::win::CarteroWindow;

mod imp {
    use glib::subclass::{object::ObjectImpl, types::ObjectSubclass};
    use gtk4::prelude::*;
    use gtk4::subclass::prelude::ObjectSubclassExt;
    use gtk4::subclass::{application::GtkApplicationImpl, prelude::ApplicationImpl};

    #[derive(Default)]
    pub struct CarteroApplication;

    #[glib::object_subclass]
    impl ObjectSubclass for CarteroApplication {
        const NAME: &'static str = "CarteroApplication";
        type Type = super::CarteroApplication;
        type ParentType = gtk4::Application;
    }

    impl ObjectImpl for CarteroApplication {}

    impl ApplicationImpl for CarteroApplication {
        fn activate(&self) {
            self.obj().get_window().present();
        }
    }

    impl GtkApplicationImpl for CarteroApplication {}
}

glib::wrapper! {
    pub struct CarteroApplication(ObjectSubclass<imp::CarteroApplication>)
        @extends gio::Application, gtk4::Application,
        @implements gio::ActionMap, gio::ActionGroup;

}

impl CarteroApplication {
    pub fn new() -> Self {
        Object::builder()
            .property("application-id", "es.danirod.Cartero")
            .build()
    }

    pub fn get_window(&self) -> CarteroWindow {
        CarteroWindow::new(&self)
    }
}
