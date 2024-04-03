use glib::Object;

mod imp {
    use std::cell::RefCell;

    use glib::object::ObjectExt;
    use glib::Properties;
    use gtk4::glib;
    use gtk4::glib::subclass::prelude::*;

    #[derive(Default, Debug, Properties)]
    #[properties(wrapper_type = super::Header)]
    pub struct Header {
        #[property(get, set)]
        active: RefCell<bool>,

        #[property(get, set)]
        header_name: RefCell<String>,

        #[property(get, set)]
        header_value: RefCell<String>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for Header {
        const NAME: &'static str = "CarteroHeader";
        type Type = super::Header;
    }

    #[glib::derived_properties]
    impl ObjectImpl for Header {
        fn constructed(&self) {
            self.parent_constructed();
        }
    }
}

glib::wrapper! {
    pub struct Header(ObjectSubclass<imp::Header>);
}

impl Header {
    pub fn new(name: &str, value: &str) -> Self {
        Object::builder()
            .property("header-name", name)
            .property("header-value", value)
            .build()
    }
}

impl Default for Header {
    fn default() -> Self {
        Object::builder().build()
    }
}

#[cfg(test)]
mod tests {
    use super::Header;

    #[test]
    pub fn test_this_works() {
        let header = Header::new("Accept", "text/html");
        assert_eq!(header.header_name(), "Accept");
        assert_eq!(header.header_value(), "text/html");
        assert!(!header.active());

        header.set_header_name("Content-Type");
        assert_ne!(header.header_name(), "Accept");
        assert_eq!(header.header_name(), "Content-Type");
        header.set_active(true);
        assert!(header.active());
    }
}
