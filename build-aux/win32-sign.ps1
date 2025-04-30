param(
        [Parameter(Mandatory)]
        [String]$identity,
        [Parameter(Mandatory)]
        [String]$path
)

signtool sign /n "$identity" /t http://time.certum.pl /fd sha1 /v "$path"
signtool sign /n "$identity" /tr http://time.certum.pl /fd sha256 /td sha256 /as /v "$path"
