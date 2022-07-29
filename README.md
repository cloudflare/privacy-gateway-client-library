# Sample App Relay Library

This project contains a sample library that exposes [OHTTP](https://datatracker.ietf.org/doc/html/draft-ietf-ohai-ohttp-02) client-side interfaces for iOS and macOS. It is compatible with the [app relay](XXX) and corresponding [gateway](XXX).

## Build instructions

Prior to building for any platform, [download and install Rust](https://rustup.rs) - we recommend at least `rustc 1.56.0 (09c42c458 2021-10-18)` or greater.

Running `cargo build --release` will build the library for the current platform, and library files will appear in `./target/release/`

## iOS Build Instructions

To build the library for iOS:

```sh
# Ensure you have the XCode SDK installed and available on your PATH:
➜  xcrun --show-sdk-build-version
21A344

# Install the iOS and macOS cross-compilation targets in your Rust toolchain
➜  rustup target add aarch64-apple-ios
info: component 'rust-std' for target 'aarch64-apple-ios' is up to date

# Install the iOS and macOS cross-compilation targets in your Rust toolchain
➜  rustup target add x86_64-apple-darwin
info: component 'rust-std' for target 'x86_64-apple-darwin' is up to date
```

With the pre-requisites installed, run `cargo build` in `--release` mode:

```sh
➜  cargo build --target aarch64-apple-ios --release`
```

Library files will appear in `./target/aarch64-apple-ios/release`. SystemConfiguration.framework (part of the XCode SDK) is required for iOS and macOS apps.

## macOS Build Instructions

To build for macOS, run the following:

```
$ cargo build --target x86_64-apple-darwin --release
```

As with iOS, link with SystemConfiguration.framework when building your macOS app.
