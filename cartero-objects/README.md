cartero-objects is the library that contains the data structures used in
the application. Users of interest of this library will be:

- The actual GUI application, because the widgets are actually bound to
  these objects in order to make it easier to display and interact with
  the objects. (For instance, to bind a value with a textbox in order to
  set the value of a field.)

- The different interoperability libraries, such as the file reader to
  read and write requests into files, or any other importer or exporter
  designed to export these data structures to a foreign format like
  cURL, OpenAPI, Postman-JSON...

- Maybe advanced users who want to create custom tools that integrate
  tightly with this application, such as a command line application that
  uses these data structures to build a command line runner for request
  files that doesn't need an user interface.

These objects are GLib objects and they have the standard GObject
capabilities, such as signals or type hierarchies. Therefore, to use this
library, familiarity with the GObject system is required. Also, anything
that depends on this crate will also depend on glib, so glib-dev has to
be installed to compile this crate, and glib has to be present in the
system during runtime.

## Objects in this crate

There are some high order objects that represent whole entities of the
application, such as `Request` or `Response`.

These high order objects are made of building blocks such as
`Field` rows or request methods. There are some
additional data structures to represent parts of a request, such as
`RequestBody`, which groups the complete form for a payload, including the
type and the actual payload.

In general, if you are trying to interact with a request or a response,
you should stick to the `Request` and `Response` entities. The rest of
the building blocks will be used as part of extracting data from the
high order object, or crafting data into the object.

## Rust structs

Before the whole project moved to workspaces and this crate was created,
this application used standard Rust structs, and then had a lot of
transformation functions that could convert from Rust structs, to the
user interface state, and viceversa.

As part of the refactor, these GObject classes were created, and
currently they are tightly coupled to GLib. **You must have the development
libraries of glib on your system to compile this lib or against this lib.
You must have the runtime libraries of glib on your system to use this
lib**.

In the future, it would be good to explore the usage of the `member``
attribute of [GLib properties], This would allow to split each object
in a pure Rust struct that is then assembled into a GObject class in
order to have the advantages of a GObject class in the user interface
(pseudo-reactive properties and signals in general).

[GLib properties]: https://gtk-rs.org/gtk-rs-core/stable/latest/docs/glib/derive.Properties.html#supported-property-attributes
