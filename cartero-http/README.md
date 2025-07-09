cartero-http is the base library for working with the BoundRequest type, which
is the kind of request that has been already processed and converted into a
normalized HTTP request.

You can use cartero-http as the basis to write an actual HTTP client that
sends a Request object and gets a Response, or you can use cartero-http to
write request exporters, such as "export to cURL", "export to HAR",
"export to code"...
