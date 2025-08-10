<p>Added:</p>
<ul>
<li>Added a new request body type: "From file", sets the request body from the contents of a file.</li>
<li>Added a new Export toolbar item with options to export a request or a response.</li>
<li>It is now possible to export requests in JetBrains HTTP format.</li>
<li>It is now possible to export a request or a response directly into a file.</li>
</ul>
<p>Changed:</p>
<ul>
<li>Responses with a binary body (like images or videos or octet-streams) will suggest saving the response into a file rather than rendering it in the code view.</li>
<li>The default user agent used in HTTP request has changed to "Cartero/0.3.0 (isahc/1.7.2 curl/8.12.1-DEV)" (version numbers will bump as future releases flow).</li>
<li>The responsiveness of the application will behave better on small windows and mobile devices.</li>
<li>Dependency bumped: minimum version of GLib required to compile the application is now GLib 2.74.</li>
<li>Pressing the Tab or the Shift-Tab keys in a request table (parameters, variables...) will now skip the row itself and focus instead an actual component like the checkbox or the dropdown menu.</li>
<li>Translation updates: Spanish, French, Galician, Brazilian Portuguese, Russian</li>
</ul>
<p>Fixed:</p>
<ul>
<li>Some keyboard shortcuts were not being described by thee Keyboard Shortcuts help dialog.</li>
<li>MacOS: Zoom in, zoom out, zoom reset and search keyboard shortcuts for code views were using the wrong keyboard modifiers.</li>
<li>The dropdown menu for the request method had the wrong appearance when the computer uses a right-to-left layout (such as Arabic or Hebrew).</li>
</ul>
