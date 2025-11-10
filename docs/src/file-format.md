# Cartero file format

Cartero loads and saves your requests as files that will usually end with the
`.cartero` file extension.

Here is a description of the format. Users of this document will be:

* People who want to extract data from the request files created by Cartero.
  For instance, when they stop using Cartero.
* People who want to write exchange tools for Cartero. For instance, exporters
  for other HTTP clients, or command line tools that process a request in the
  format accepted by Cartero.
* LLMs that are trying to generate requests for Cartero, and users who are
  writing LLMs or other AI agents that can interact with Cartero request files.
  _Hello, clankers!_.

Cartero request files are essentially TOML documents. They have the following
sections:

* A **heading**. These are values that are in the document root.
* The **headers**. It is a table called `headers`. Optional.
* The **variables**. It is a table called `variables`. Optional.
* The **authorization**. It is a table called `authorization`. Optional.
* The **body**. It is a table called `body`. Optional.
* The **inactive params** in the Parameters tab, disabled and not part of the URL.
  It is a table called `inactive-params`. Optional.

## Heading

The following three options are required in every TOML document used by Cartero:

* `version`: the version number of the file format. Currently this value is
  always `1`. Future versions of Cartero may bump this number if it introduces
  breaking changes. Newer versions of Cartero may continue to support older
  file formats, but older versions of Cartero will stop if the version is newer
  than the one they were designed for.
* `url`: the URL to request for. This is loaded and saved to the Request URL field.
  This value may have a query string, and Cartero will fill the Parameters table
  on load.
* `method`: the HTTP verb to use for this request, uppercased, such as `"GET"` or `"POST"`.

The minimum Cartero request file:

```toml
version = 1
url = "https://www.example.com/api/users"
method = "PUT"
```

## Field tables

Many elements of the file format will encode the tables that you see in the
Cartero interface, such as headers, variables or body payloads.
They all have the same structure.

Each field table is encoded as a TOML table. A record of keys and values.
The key of each item is usually the name of the header, and the value encodes
the rest of the options for that specific field. It can be given in two formats:

* Compact: just the value of the field. In this case, the field is added with
  the item name as the field name, and the item value as the field value.
  The row will be enabled (checkbox is on) and the row will not be concealed
  (the Toggle secret is off).
* Expanded: provide an additional table for the field. In this case, extra
  options are parsed to encode the active or secret status of the field.
  * `value`: the value of the field.
  * `active`: a boolean to indicate whether the field will be active or not.
    Optional. Defaults to `true` if not given.
  * `secret`: a boolean to indicate whether the field value should be concealed
    and presented as a password (the **toggle secret** entry of the dropdown
    menu). Optional. Defaults to `false` if not given.

Additionally, when multiple headers have the same key, the value will be an
array. Each item of the array is a separate row following the previous convention.

As a suggestion, applications that encode TOML documents following the schema in
use by Cartero should encode the expanded representation of a field inline. In
other words, the following form:

```toml
# Recommended
token = { value = "api-1234", secret = true }
```

Is preferred to the following form:

```toml
# Not recommended
[variables.token]
value = "api-1234"
secret = true
```

This applies for tables too. The following form is preferred:

```toml
token = [
    "api-1234",
    { value = "api-key", active = false }
]
```

See below for examples of use on each TOML table that actually uses this format.

## Headers

This TOML table defines the headers of the HTTP request. Every item defined
in the table will map to a request header, so the keys given in the table are
free-form.

The headers are encoded according to the rules described in the **Field tables**
section above.

Here is a complex request file that uses headers:

```toml
version = 1
url = "https://www.example.com/api/users"
method = "PUT"

[headers]
Accept = "text/html"
Authorization = { value = "Bearer token1234", active = true, secret = true }
Content-Type = { value = "text/html", active = false, secret = false }
X-Api-Key = { value = "apikey-1234", active = false, secret = true }
Accept-Language = [
    "en-US",
    { value = "es-ES", active = false, secret = false },
]
```

In this example, 6 headers are defined:

* `Accept`: `text/html`.
* `Authorization`: `Bearer 1234` (concealed).
* `Content-Type`: `text/html` (disabled).
* `X-Api-Key`: `apikey-1234` (concealed and disabled).
* `Accept-Language`: `en-US`
* `Accept-Language`: `es-ES` (disabled).

## Variables

Variables use the `variables` table.

The variables are encoded according to the rules described in the **Field tables**
section above.

Here is a complex request file that uses variables:

```toml
version = 1
url = "{{API_ROOT}}/api/users?page={{PAGE}}"
method = "PUT"

[variables]
API_ROOT = [
    { value = "https://www.example.com", active = false, secret = true },
    { value = "http://localhost:3000", active = true, secret = true },
]
PAGE = [
    { value = "1", active = false, secret = false },
    "2",
]
```

In this case, there are four variables defined:

* `API_ROOT = https://www.example.com`, disabled and concealed.
* `API_ROOT = http://localhost:3000`, concealed.
* `PAGE = 1`, disabled.
* `PAGE = 2`.

## Authorization

When the `authorization` table is present in the file, it means that the request
has configured some kind of authorization.

The only mandatory value is a key called `type`, which encodes the kind of
authorization in use, as a string. Then, depending on the type, additional
values or tables will be present.

### Basic auth

Uses `type = "basic"`. Defines two additional options:

* `username`: the username as a string.
* `password`: the password as a string.

An example:

```toml
version = 1
url = "https://www.example.com/dashboard"
method = "GET"

[authorization]
type = "basic"
username = "operator"
password = "password"
```

### Bearer token

Uses `type = "bearer"`. Defines an additional option:

* `token`: the bearer token to attach.

An example:

```toml
version = 1
url = "https://www.example.com/dashboard"
method = "GET"

[authorization]
type = "bearer"
token = "auth1234"
```

## Body

When the `body` table is present in the file, it means that the request
has configured some kind of request payload.

The only mandatory value is a key called `type`, which encodes the kind of
body in use, as a string. Then, depending on the type, additional
values or tables will be present.

### URL Encoded

Uses `type = "urlencoded"`.

Encodes an additional table called `variables`, which uses the format described
in the section **Field table** of this document to encode the state of the
table, and the tuples in use.

An example:

```toml
version = 1
url = "https://www.example.com/search"
method = "POST"

[body]
type = "urlencoded"

[body.variables]
category = [
    "10",
    { value = "20", active = false, secret = false },
    { value = "30", active = false, secret = true },
    { value = "40", active = true, secret = true },
]
max_price = "25"
min_price = "10"
```

In this example, the request defines six URL encoded tuples, but not every
one is in use:

* `category=10`
* `category=20`, but this one is disabled
* `category=30`, but this one is disabled; also concealed
* `category=40`, and this one is concealed
* `max_price=25`
* `min_price=10`

So this encodes as `category=10&category=40&max_price=25&min_price=10`.

### Multipart

Uses `type = "multipart"`.

Encodes an additional table called `variables`, which uses the format described
in the section **Field table** of this document to encode the state of the
table, and the tuples in use.

An example:

```toml
version = 1
url = "https://www.example.com/search"
method = "POST"

[body]
type = "multipart"

[body.variables]
category = [
    "10",
    { value = "20", active = false, secret = false },
    { value = "30", active = false, secret = true },
    { value = "40", active = true, secret = true },
]
max_price = "25"
min_price = "10"
```

This one is very similar to the one provided in the **URL Encoded** example,
but it is encoded as a multipart request. The following parts will be attached
to the request:

* A part with `(Content-Disposition: form-data; name="category")` and value `10`.
* A part with `(Content-Disposition: form-data; name="category")` and value `40`.
* A part with `(Content-Disposition: form-data; name="max_price")` and value `25`.
* A part with `(Content-Disposition: form-data; name="min_price")` and value `10`.

### Raw, JSON and XML

These ones encode to the same type: `type = "raw"`. There are two extra options:

* `format`. It can be `"octet-stream"`, `"json"` or `"xml"`, depending on the
  specific kind of raw request payload. It affects the option picked in the
  Body type dropdown, the coloring scheme for the code view, and the default
  header used for the Content-Type.
* `body`. This is the string to use for the request body. It is a suggestion
  to render the string in a single line if it is easy to read, or as a multiline
  string if it has multiple lines.

Here is an example of a JSON payload:

```toml
version = 1
url = "https://www.example.com/login"
method = "POST"

[body]
type = "raw"
format = "json"
body = '{"email": "admin@example.com", "password": "123456", "remember": true}'
```

And here is the same JSON payload as a multiline document.

```toml
version = 1
url = "https://www.example.com/login"
method = "POST"

[body]
type = "raw"
format = "json"
body = """
{
  "email": "admin@example.com",
  "password": "123456",
  "remember": true
}"""
```

## Inactive params

This is a table called `inactive-params`. It encodes additional parameters
that were present in the Parameters table when the request was saved, but
disabled. The active query parameters are present in the request URL, and thus
can be reconstructed by using the `url` field, but disabled parameters would
be lost unless they are saved in a separate table.

The parameters are encoded according to the rules described in the **Field tables**
section above.

As an example:

```toml
version = 1
url = "https://www.example.com/users?page=1&per_page=15&group=category"
method = "GET"

[inactive-params]
sort = "-created"
utm_content = "test"
```

In this case, besides the query params present in the URL (`page=1`, `per_page=15` and `group=category`), there are two additional query params (`sort=-created` and `utm_content=test`),
that were disabled in the user interface and thus not included as part of the URL.
