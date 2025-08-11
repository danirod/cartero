<p>Added:</p>
<ul>
    <li>An authorization tab, currently supporting basic authentication and bearer tokens.</li>
    <li>A Cancel button to stop an HTTP request in progress.</li>
    <li>An error panel to report errors related to a failing web request.</li>
    <li>Alert dialogs to report errors related to loading and saving files.</li>
    <li>New keyboard shortcuts and mouse gestures for zooming text views.</li>
    <li>Improved the about dialog.</li>
</ul>
<p>Changed:</p>
<ul>
    <li>Application errors will now properly report the cause of an error and not just generic messages.</li>
    <li>During a request, the application will now stay clickable and not freeze.</li>
    <li>Units for the response size will now be internationalized (for instance, 32.4 Ko rather than 32.4 kB when running in French).</li>
    <li>Reduced the precission of the response duration indicator to prevent confusion.</li>
    <li>Disabled query params in the Parameters table will now be persisted into the file.</li>
    <li>Simplified the application icon and updated the branding.</li>
</ul>
<p>Fixed:</p>
<ul>
    <li>Requests whose URL do not start with http:// or https:// (such as "localhost:3000/users") should not fail now.</li>
    <li>During prettification of JSON responses, objects were being sorted; they will respect the original order now.</li>
    <li>The headers and variables tables lost the ability to report when a field name was duplicated.</li>
    <li>Disabled query params in the Parameters table were lost when the request URL changed.</li>
    <li>Windows: closing the settings dialog sometimes buried the Cartero main window under other windows.</li>
    <li>Windows: the title bar stayed in light mode even when the application ran in dark mode.</li>
</ul>
