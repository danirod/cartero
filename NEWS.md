# News file for Cartero

These are the user-visible changes noticeable within Cartero.

## [26.1] - unreleased

### Changed

- - Updated Keyboard Shortcuts menu to use the new Adwaita based dialog
    where possible - thanks, @youpie.
- - Updated Rust project dependencies.
- - Updated codebase Rust edition to Rust 2024.

### Translation Updates

- Galician
- Indonesian
- Polish

## [26.0] - 2026-01-25

### Added

- Added a color scheme selector for the code views.
- Added application window theming and tinting.
- Added a menu option to duplicate requests.

### Changed

- The response pane will now render the effective URL of a response.
- It is now possible to reveal the values of .env variables in the
  endpoint pane.

### Fixed

- Responses were being treated as binary if they contained the CR (\\r)
  character.
- Disabling TLS validation still failed a request if the certificate
  didn't have a matching SAN.
- MacOS: the main application window could sometimes stop receiving
  mouse clicks on macOS 26.

### Translation Updates

- Basque
- Indonesian
- Portuguese (Brazil)
- Chinese (Simplified)

## [25.0] - 2025-11-14

### Added

- Added support for proxies.
- Added support for dotenv files: values defined in an .env file can be
  loaded as variables.
- The Headers pane will now present pre-generated headers added by
  Cartero.
- Added a new request body type: "From file", loads the raw request
  body from the contents of a file.
- It is now possible to export requests in JetBrains HTTP format.
- It is now possible to export a request or a response directly into a
  file.
- It is now possible to export the contents of an HTTP response.
- The welcome pane now has an additional sheet with information and
  links.

### Changed

- The Export tab has moved to a dropdown menu in the window toolbar.
- Opening a new endpoint will automatically focus the URL entry field.
- Disabled variables, headers or parameters will now render slightly
  dimmed.
- When multiple variables or headers with the same key are present,
  those who will be ignored be marked.
- Showing or concealing passwords in basic auth or bearer token fields
  will now be remembered in the application state.
- The settings dialog has a new look and feel, using split windows.
- Responses with a binary body (like images or videos or octet-streams)
  will suggest saving the response into a file rather than rendering it
  in the code view.
- The default user agent used in HTTP request has changed to
  "Cartero/25.0 (isahc/1.7.2 curl/8.12.1-DEV)" (version numbers will
  bump as future releases flow).
- The responsiveness of the application will behave better on small
  windows and mobile devices.
- Dependencies bumped: Cartero now requires at least GNOME SDK 46 to
  build the app.
- Windows: the application look and feel on Windows 11 now uses the
  acrilyic look and feel.

### Fixed

- HTTP responses sometimes were not rendered if they had the
  Content-Type header set to JSON but they were not actually parseable
  as JSON.
- HTTP response headers sometimes did not render properly when they had
  angle brackets (such as the Link header).
- HTTP requests sometimes could not be exported when they had spaces in
  the URL.
- Windows: overriding the language in the settings dialog previously
  had no effect.
- MacOS: the URL bar did not focus when pressing Command-L.
- MacOS: the export dialog did not actually copy the code to the system
  clipboard.

### Translation Updates

- Catalan
- Czech
- German
- Esperanto
- Spanish
- Basque
- French
- Galician
- Indonesian
- Portuguese
- Portuguese (Brazil)
- Romanian
- Russian
- Tamil

## [0.2.4] - 2025-08-10

### Changed

- Pressing the Tab or the Shift-Tab keys in a request table
  (parameters, variables...) will now skip the row itself and focus
  instead an actual component like the checkbox or the dropdown menu.
- MacOS: on systems with a dual graphics card setup, Cartero will now
  run on the integrated rather than the discrete one. (This is good, it
  will save a lot of battery.)

### Fixed

- Fixed accessibility issues: missing labels and tooltips within some
  window widgets.
- Some keyboard shortcuts were not being described by thee Keyboard
  Shortcuts help dialog.
- The dropdown menu for the request method had the wrong appearance
  when the computer uses a right-to-left layout (such as Arabic or
  Hebrew).
- MacOS: Zoom in, zoom out, zoom reset and search keyboard shortcuts
  for code views were using the wrong keyboard modifiers.

### Translation Updates

- Catalan
- Czech
- German
- Esperanto
- Spanish
- Basque
- French
- Galician
- Portuguese
- Portuguese (Brazil)
- Romanian
- Russian
- Tamil

## [0.2.3] - 2025-07-05

### Fixed

- Exporting a request as cURL did not include the request payload when
  the type is XML.

### Translation Updates

- Catalan
- German
- Basque
- Portuguese (Brazil)
- Russian
- Tamil

## [0.2.2] - 2025-04-30

### Changed

- The URL field will be less laggy when the URL has query parameters,
  thanks to a performance increase in how query params are reflected in
  the table in the Parameters tab.
- Windows, macOS, AppImage: updated vendored runtime to GTK 4.18.4.

### Fixed

- The query params table did not reflect the query params when the
  address in the URL field started with a variable.
- Fields modified in the query parameters table will not be rendered
  url-encoded in the URL field, as they used to.
- Requests will not fail anymore if they have a disabled header that
  references an undefined variable.
- GNU/Linux: fixed an application crash if gsettings-desktop-schemas is
  not available.
- macOS: the save dialog will not present two extensions in the initial
  file name in macOS Sequoia.

### Translation Updates

- Czech
- German
- Basque
- Portuguese
- Tamil

## [0.2.1] - 2025-04-03

### Fixed

- Disabled variables in the variables table were still being picked

### Translation Updates

- Catalan
- Spanish
- Galician
- Portuguese (Brazil)

## [0.2.0] - 2025-03-21

### Added

- An authorization tab, currently supporting basic authentication and
  bearer tokens.
- A Cancel button to stop an HTTP request in progress.
- An error panel to report errors related to a failing web request.
- Alert dialogs to report errors related to loading and saving files.
- New keyboard shortcuts and mouse gestures for zooming text views.
- Improved the about dialog.
- Windows, MacOS and AppImage: a software update checker has been added
  to the settings dialog.
- MacOS: added a menu bar for quick access to the application commands.

### Changed

- Application errors will now properly report the cause of an error and
  not just generic messages.
- During a request, the application will now stay clickable and not
  freeze.
- Units for the response size will now be internationalized (for
  instance, 32.4 Ko rather than 32.4 kB when running in French).
- Reduced the precission of the response duration indicator to prevent
  confusion.
- Disabled query params in the Parameters table will now be persisted
  into the file.
- Simplified the application icon and updated the branding.
- MacOS and Windows: the default application font will now be larger.
- MacOS: the window now uses an integrated title bar like the GNU/Linux
  version.
- Windows: cartero.exe now uses the application icon for the executable
  file.
- Windows: require Windows 10 or newer to install the application.

### Fixed

- Requests whose URL do not start with http:// or https:// (such as
  "localhost:3000/users") should not fail now.
- During prettification of JSON responses, objects were being sorted;
  they will respect the original order now.
- The headers and variables tables lost the ability to report when a
  field name was duplicated.
- Disabled query params in the Parameters table were lost when the
  request URL changed.
- Windows: closing the settings dialog sometimes buried the Cartero
  main window under other windows.
- Windows: the title bar stayed in light mode even when the application
  ran in dark mode.

### Translation Updates

- Catalan
- Czech
- Esperanto
- Spanish
- French
- Portuguese (Brazil)
- Romanian
- Russian
- Tamil

## [0.1.5] - 2025-02-15

### Changed

- Changed the error message when a request uses an unsupported protocol
  for clarity purposes.

### Fixed

- Prevents tabs from becoming unresponsive if a request fails in
  certain conditions.
- Prevents requests from failing if they have a trailing or leading
  space.
- Requests whose URL start with a variable (bound to something that
  starts with http or https) can be made again.

### Translation Updates

- Czech
- French
- Portuguese (Brazil)

## [0.1.4] - 2025-01-26

### Fixed

- Sending an HTTP request failed if the protocol was not specified.
- Pre-compiled versions for Windows were not signed.

### Translation Updates

- Czech
- French

## [0.1.3] - 2024-12-23

### Added

- Added a tab to generate a cURL command with the contents of a
  request.
- Added a preferences dialog to control application settings.
- Added a search functionality (Ctrl-F) to the text area panes.

### Changed

- It is now possible to disable validation of TLS certificates.
- It is now possible to follow redirections when making HTTP requests.
- It is now possible to configure a timeout for the HTTP request.
- It is now possible to customize the font used for the text area
  panes.
- It is now possible to customize the light or dark appearance of the
  application.

### Fixed

- Fixed the HTTP response label sometimes having the wrong semantic
  color.

### Translation Updates

- Catalan
- Esperanto
- Spanish
- Romanian
- Russian
- Tamil

## [0.1.2] - 2024-10-11

This is a minor release that addresses some issues and fixes some
things found.

### Changed

- It is now possible to open multiple files using the dialog picker.
- Translation updates: Russian

### Fixed

- Fix translation for the about and shortcuts dialog.
- Word wrapping of long lines without spaces did not work properly.
- Deactivate toolbar buttons if no request are open.

### Translation Updates

- Spanish
- Russian

## [0.1.1] - 2024-07-30

This is a minor release that addresses some issues and small changes
found in the last couple of days. It accepts feedback from the
community and even some pull requests received in the last days.

### Changed

- Pressing the Enter key while focusing the request URL entry will now
  send the HTTP request
- The response body page is now the default page for the response
  notebook
- The HTTP status code will now use semantic colors to report the
  status code category (success, client error, server error...)
- Provided a Metainfo file for submission into Flathub

### Fixed

- The application may not open files when running as a Flatpak in
  sandbox mode
- The Nix flake did not build due to some missing dependencies
- Clicking on any link on Microsoft Windows did not open the default
  web browser

### Translation Updates

- Catalan
- Esperanto
- Spanish
- Romanian

## [0.1.0] - 2024-07-26

Initial release. I've crafted a MVP that consolidates the most
important features to start using Cartero. Some features has been
delayed for a future release, but there is already enough features for
it to be useful.

### Added

- A fully functional HTTP client with support for multiple request
  methods, and payload types.
- A variable engine that allows to move things such as API keys,
  passwords or hostnames into a variable that can be injected later
  into the URL or other headers.
- File support to load and store requests for a future session.

### Translation Updates

- Esperanto
- Spanish
