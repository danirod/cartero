<p>Added:</p>
<ul>
  <li>Added support for proxies.</li>
  <li>
    Added support for dotenv files: values defined in an .env file can be loaded
    as variables.
  </li>
  <li>
    The Headers pane will now present pre-generated headers added by Cartero.
  </li>
  <li>
    Added a new request body type: "From file", loads the raw request body from
    the contents of a file.
  </li>
  <li>It is now possible to export requests in JetBrains HTTP format.</li>
  <li>
    It is now possible to export a request or a response directly into a file.
  </li>
  <li>It is now possible to export the contents of an HTTP response.</li>
  <li>
    The welcome pane now has an additional sheet with information and links.
  </li>
</ul>
<p>Changed:</p>
<ul>
  <li>The Export tab has moved to a dropdown menu in the window toolbar.</li>
  <li>Opening a new endpoint will automatically focus the URL entry field.</li>
  <li>
    Disabled variables, headers or parameters will now render slightly dimmed.
  </li>
  <li>
    When multiple variables or headers with the same key are present, those who
    will be ignored be marked.
  </li>
  <li>
    Showing or concealing passwords in basic auth or bearer token fields will
    now be remembered in the application state.
  </li>
  <li>The settings dialog has a new look and feel, using split windows.</li>
  <li>
    Responses with a binary body (like images or videos or octet-streams) will
    suggest saving the response into a file rather than rendering it in the code
    view.
  </li>
  <li>
    The default user agent used in HTTP request has changed to "Cartero/25.0
    (isahc/1.7.2 curl/8.12.1-DEV)" (version numbers will bump as future releases
    flow).
  </li>
  <li>
    The responsiveness of the application will behave better on small windows
    and mobile devices.
  </li>
  <li>
    Dependencies bumped: Cartero now requires at least GNOME SDK 46 to build the
    app.
  </li>
  <li>
    Windows: the application look and feel on Windows 11 now uses the acrilyic
    look and feel.
  </li>
  <li>
    Translation updates: Czech, Basque, Galician, Indonesian, Portuguese,
    Brazilian Portuguese, Spanish, Russian, Ukrainian.
  </li>
</ul>
<p>Fixed:</p>
<ul>
  <li>
    HTTP responses sometimes were not rendered if they had the Content-Type
    header set to JSON but they were not actually parseable as JSON.
  </li>
  <li>
    HTTP response headers sometimes did not render properly when they had angle
    brackets (such as the Link header).
  </li>
  <li>
    HTTP requests sometimes could not be exported when they had spaces in the
    URL.
  </li>
  <li>
    Windows: overriding the language in the settings dialog previously had no
    effect.
  </li>
  <li>MacOS: the URL bar did not focus when pressing Command-L.</li>
  <li>
    MacOS: the export dialog did not actually copy the code to the system
    clipboard.
  </li>
</ul>
