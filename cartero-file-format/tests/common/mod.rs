use cartero_objects::Field;

pub fn assert_field(field: &Field, name: &str, value: &str, active: bool, masked: bool) {
    assert_eq!(field.key(), name);
    assert_eq!(field.value(), value);
    assert_eq!(field.active(), active);
    assert_eq!(field.masked(), masked);
}
