uv run ./download_sdk.py

# TODO: https://github.com/rust-lang/rust-bindgen/issues/2535
bindgen ../target/sdk/everything_plugin.h -o src/sys.rs `
    --allowlist-file '../target/sdk/everything_plugin.h' `
    --override-abi '.*=system' `
    --raw-line '//! Reference: https://www.voidtools.com/forum/viewtopic.php?t=16535' `
    --raw-line '#![allow(non_snake_case, non_camel_case_types, dead_code)]' `
    --merge-extern-blocks `
    --rustified-enum .* `
    --verbose
