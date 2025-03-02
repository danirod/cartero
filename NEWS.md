# News file for Cartero

## [0.1.3] - 2024-12-23

### Added

- Added a tab to generate a cURL command with the contents of a request.
- Added a preferences dialog to control application settings.
- Added a search functionality (Ctrl-F) to the text area panes.

### Changed

- It is now possible to disable validation of TLS certificates.
- It is now possible to follow redirections when making HTTP requests.
- It is now possible to configure a timeout for the HTTP request.
- It is now possible to customize the font used for the text area panes.
- It is now possible to customize the light or dark appearance of the application.
- Translation updates: Spanish, Tamil.

### Fixed

- Fixed the HTTP response label sometimes having the wrong semantic color.

## [0.1.2] - 2024-10-11

This is a minor release that addresses some issues and fixes some things found.

### Changed

- It is now possible to open multiple files using the dialog picker.
- Translation updates:
  - Russian

### Fixed

- Fix translation for the about and shortcuts dialog.
- Word wrapping of long lines without spaces did not work properly.
- Deactivate toolbar buttons if no request are open.

## [0.1.1] - 2024-27-30

This is a minor release that addresses some issues and small changes found in the last couple of days. It accepts feedback from the community and even some pull requests received in the last days.

### Changed

- Pressing the Enter key while focusing the request URL entry will now send the HTTP request
- The response body page is now the default page for the response notebook
- The HTTP status code will now use semantic colors to report the status code category (success, client error, server error...)
- Provided a Metainfo file for submission into Flathub
- Translation updates
  - Catalan
  - Romanian
  - Spanish

### Fixed

- The application may not open files when running as a Flatpak in sandbox mode
- The Nix flake did not build due to some missing dependencies
- Clicking on any link on Microsoft Windows did not open the default web browser

## [0.1.0] - 2024-07-26

Initial release. I've crafted a MVP that consolidates the most important features to start using Cartero. Some features has been delayed for a future release, but there is already enough features for it to be useful.

### Added

- A fully functional HTTP client with support for multiple request methods, and payload types.
- A variable engine that allows to move things such as API keys, passwords or hostnames into a variable that can be injected later into the URL or other headers.
- File support to load and store requests for a future session.
