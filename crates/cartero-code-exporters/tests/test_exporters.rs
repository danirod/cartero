// Copyright 2024-2026 the Cartero authors
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.
//
// SPDX-License-Identifier: GPL-3.0-or-later

use cartero_code_exporters::{export_request, Format};

macro_rules! test_suites {
    (
        formats [
            $( $mod_name:ident => { directory: $mod_dir:literal, extension: $mod_ext:literal, value: $mod_value:path } ),+ $(,)?
        ],
        cases $cases:tt $(,)?
    ) => {
        $(
            mod $mod_name {
                use super::*;

                test_suites!(@modloop $mod_dir, $mod_ext, $mod_value, $cases);
            }
        )*
    };

    (@modloop
        $mod_dir:literal, $mod_ext:literal, $mod_value:path,
        [ $( $file:literal => $name:ident ),+ $(,)? ]
    ) => {
        $(
            #[test]
            fn $name() {
                let input = include_str!(concat!("cartero/", $file, ".cartero"));
                let expected = include_str!(concat!($mod_dir, "/", $file, $mod_ext));
                let request = cartero_file_format::deserialize_request(&input)
                    .unwrap()
                    .object()
                    .clone();
                let actual = export_request($mod_value, &request).expect("Couldn't export request");
                assert_eq!(actual, expected.trim_end_matches('\n'));
            }
        )+
    };
}

test_suites! {
    formats [
        curl => { directory: "curl", extension: ".curl", value: Format::Curl },
        ijhttp => { directory: "ijhttp", extension: ".http", value: Format::Ijhttp },
    ],
    cases [
        "auth_basic" => auth_basic,
        "auth_bearer" => auth_bearer,
        "body_file" => body_file,
        "body_file_with_content_type" => body_file_with_content_type,
        "body_json" => body_json,
        "body_multiline_json" => body_multiline_json,
        "body_multipart" => body_multipart,
        "body_multipart_combined" => body_multipart_combined,
        "body_multipart_duplicate" => body_multipart_duplicate,
        "body_octet_stream" => body_octet_stream,
        "body_url_encoded" => body_url_encoded,
        "body_url_encoded_combined" => body_url_encoded_combined,
        "body_url_encoded_duplicate" => body_url_encoded_duplicate,
        "body_xml" => body_xml,
        "complex" => complex,
        "dupe_headers" => dupe_headers,
        "headers" => headers,
        "quotes_multipart" => quotes_multipart,
        "quotes_raw" => quotes_raw,
        "quotes_urlencoded" => quotes_urlencoded,
        "simple_request" => simple_request,
        "variables" => variables,
    ],
}
