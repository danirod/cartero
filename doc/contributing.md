# Contributing

Thank you for your interest in helping Cartero grow! This document will
provide information and tips on how to help the project.

## Contributing with code

**If you plan on contributing to the project**, use the development profile.
It will also configure a Git hook so that the source code is checked prior to
authoring a Git commit. The hook runs `cargo fmt` to assert that the code is
formatted. Read `hooks/pre-commit.hook` to inspect what the script does.

```sh
meson setup build -Dprofile=development
```

This project is highly appreciative of contributions. If you know about Rust,
GTK or the GNOME technologies and want to help during the development, you can
contribute if you wish. [Fork the project][fork] and commit your code.

Some checklist rules before submitting a pull request:

- **Use a feature branch**, do not make your changes in the trunk branch
  directly.

- **Rebase your code** and make sure that you are working on top of the most
  recent version of the trunk branch, in case something has changed while you
  were working on your code.

- **Update the locales** if you changed strings. The ninja target that you are
  looking for is called `cartero-update-po` (such as `ninja -C build
cartero-update-po`). Don't worry, you don't have to translate the strings by
  yourself, but make sure that the new templates are added to the .po and .pot
  files.

- **Use the pre-commit hook**. The pre-commit hook will validate that your code
  is formatted. It should be automatically configured if you run Meson in
  development mode (`-Dprofile=development`), but you can install it on your
  own or run `hooks/pre-commit.hook`.

The project is starting small, so if you want to do something big, it is best
to first start a discussion thread with your proposal in order to see how to
make it fit inside the application.

This project is published with the GNU General Public License 3.0 or later.
Therefore, your contribution will be checked out into the repository under that
license. **Make sure you are comfortable with the license before contributing**.
Specifically, while you retain copyrights of your contribution, you acknowledge
that you allow anyone to use, study or distribute any code you write under the
terms of that license.

This application is not affiliated with the GNOME project, but we follow the
[GNOME Code of Conduct][coc] anyway as a guideline and we expect you to follow
it when interacting with the repository.

## Contributing with translations

Do you want to use Cartero in your language? We are using [Weblate][weblate]
to coordinate and translate comfortably this project using a web interface.
Make an account and start proposing strings and they will be added to the
application. That will also entitle you as a contributor!

## Contributing with feedback

Cartero is still getting new features, and hopes to be as useful as it can be.
Found a bug or something is wrong? Report it. An use case you are missing?
Report it. Show us how you integrate Cartero on your workflow so that we can
build our diverse list of use cases.

[fork]: https://github.com/danirod/cartero/fork
[weblate]: https://hosted.weblate.org/projects/cartero/
[coc]: https://conduct.gnome.org
