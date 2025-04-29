<p>Changed:</p>
<ul>
  <li>The URL field will be less laggy when the URL has query parameters, thanks to a performance increase in how query params are reflected in the table in the Parameters tab.</li>
  <li>Translation updates: Czech, German, Basque, Portuguese, Tamil.</li>
  <li>Windows, macOS, AppImage: updated vendored runtime to GTK 4.18.4.</li>
</ul>
<p>Fixed:</p>
<ul>
  <li>The query params table did not reflect the query params when the address in the URL field started with a variable.</li>
  <li>Fields modified in the query parameters table will not be rendered url-encoded in the URL field, as they used to.</li>
  <li>Requests will not fail anymore if they have a disabled header that references an undefined variable.</li>
  <li>GNU/Linux: fixed an application crash if gsettings-desktop-schemas is not available.</li>
  <li>macOS: the save dialog will not present two extensions in the initial file name in macOS Sequoia.</li>
</ul>
