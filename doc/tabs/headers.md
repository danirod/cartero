# Headers

This tab allows you to set headers that will be included as part of the request.

The last item of the table is a placeholder that you can use to create new headers.
Start typing into the **Name** or **Value** placeholders in order to create a
new header.

![Adding headers to the header pane](../images/cartero-header-pane.png)

You can quickly disable a header without removing it by toggling the checkbox
that is next to each row. Press the checkbox again in order to enable it again.

The extra dropdown menu allows you to do a couple of things:

* Toggle secret. This will mask the value in the row to hide it with asterisks. **This is purely cosmetic**. It is designed as a way to hide values, for instance if you are screensharing your screen. However, the value will be sent as is in your HTTP requests. **Additionally, the value will be saved plain text in request files**.
* Delete. This will delete the row completely. **Does not currently ask for confirmation, does not undo at the moment**. (But it should!)

If you use [variables](./variables.md), you can add them here by placing the name of the variable between two curly brackets, like in `{{ VARIABLE }}`. Spaces are optional.

## Header limitations

Following the HTTP spec, these limitations apply.

**Cartero assumes that the order of the headers does not matter.**
(In fact, they don't). At the moment it is not possible to change the order
of the rows of the table.

It is not possible to use non-ASCII characters in a header name or value.
The HTTP spec recommends escaping the characters or encoding them using
URI encoding. Cartero will detect invalid usage of characters and report it.

![Cartero showing an error because of the value of a header](../images/cartero-invalid-header-value.png)

Cartero currently does not support multiple headers with the same name. If
there is more than one row with the same name, only the lowest value in the
table will be used, overriding any previous value.

![Duplicate headers show a remark](../images/cartero-duplicate-headers.png)

This is technically not correct, because there are some headers that accept
duplicate keys (cookies, for example). [Section 5.2 of RFC 9110](https://www.rfc-editor.org/rfc/rfc9110.html#section-5.2) covers this. Until Cartero supports this
behaviour, you can collapse every value into a single header, using a comma
as a separator.
