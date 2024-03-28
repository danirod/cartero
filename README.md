# Cartero

Make HTTP requests and test APIs.

> 🚧 This is a work in progress and therefore you should expect that the
> application may not have all the features at this moment.

## Motivation

This project exists because there aren't many native graphical HTTP testing
applications / graphical alternatives to cURL that are fully free software, and
I think the world has had enough of Electron / non-native applications that are
are anonymously accesible until one day you are forced to create an account and
log in to use.

## Roadmap

v0.1 is the first iteration and development is in progress. **The goal with
version v0.1 is to have a basic user interface to make HTTP requests
graphically**, supporting only the most basic features:

* Make HTTP requests setting the endpoint URL and the HTTP verb to use.
* Configure the payload and the request headers.
* Get the response headers, body, status code, size and duration of a request.

To achieve version 0.1, the following has to be done:

* [ ] Settle on the project structure (translations, GTK, build tools...)
* [ ] Design the user interface components (GTK 4, probably Adw 1).
* [ ] Implement the internal HTTP client and connect it to the user interface.

On future versions, more capabilities will be added:

* Support for persisting requests to re-use them in future sessions.
* Support for variables, which can be configured per environment, such as
  production, staging or development.
* Export a request as a cURL command or generate code for Axios, net/http and
  other software libraries.

## How to build

> These instructions will change very soon once Meson is introduced. Don't get
> very used to them because they will change.

Currently, to build the application you'll have to make sure that the required
libraries are installed on your system.

* glib >= 2.80
* gtk >= 4.14
* gtksourceview >= 5.12

To compile the application, run `cargo build`.

To optimize the application, run `cargo build --release`.

The binary will be generated at the proper subdirectory inside `target`, or you
can run it using `cargo run`.

Also, if you want to modify the Blueprint files, you are going to need
**blueprint-compiler >= 0.10**. You might be able to get it from your package
manager or you can [build it from sources][bp]. Invoke the compiler and make
sure you update the .ui files from the .blp source files before compiling the
application.

## Contributing

> 🐛 This project is currently a larva trying to grow. Do you want to get in?
> Take a seat!

This project is highly appreciative of contributions. If you know about Rust,
GTK or the GNOME technologies and want to help during the development, you can
contribute if you wish. [Fork the project][fork] and commit your code. **Use a
feature branch, do not make your changes in the trunk branch**. Push to your
fork and send a pull request.

The project is starting small, so if you want to do something big, it is best
to first start a discussion thread with your proposal in order to see how to
make it fit inside the application.

While this application is not official and at the moment is not affiliated with
GNOME, you are expected to follow the [GNOME Code of Conduct][coc] when
interacting with this repository.


## Credits and acknowledgments

Cartero is being developed by [Dani Rodríguez][danirod].

Big shoutout to the [contributors][contrib] who have sent patches or
translations!

Also, Christian suggested Cartero as the name for the application and I liked
it enough to call it like so, therefore shoutout to Christian as well!

Finally, shoutout to many of the GTK and GNOME Circle applications out there whose
source code I've read in order to know how to use some of the GTK features that
you cannot learn just by reading the official docs.

## Blog
Danirod's dev blog (in spanish) of Cartero [blog].

[bp]: https://gitlab.gnome.org/jwestman/blueprint-compiler
[coc]: https://conduct.gnome.org
[contrib]: https://github.com/danirod/cartero/graphs/contributors
[danirod]: https://github.com/danirod
[fork]: https://github.com/danirod/cartero/fork
[blog]: https://danirod.es/secciones/devlogs/cartero/
