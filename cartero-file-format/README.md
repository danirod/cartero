cartero-file-format is the main serialization and deserialization library for
the Cartero file formats. This allows to persist and retrieve from external
storage requests using our internal file format.

This library will be of interest of the GUI application, in order to wire the
load and save functions so that they use these functions when saving a request
to a file for later.

These functions return instances of the Cartero objects described in the
**cartero-objects** crate. Therefore, they depend on GLib to be compiled and
to work in general.
