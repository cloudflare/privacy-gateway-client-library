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
➜  cargo build --target aarch64-apple-ios --release
```

Library files will appear in `./target/aarch64-apple-ios/release`. SystemConfiguration.framework (part of the XCode SDK) is required for iOS and macOS apps.

## macOS Build Instructions

To build for macOS, run the following:

```sh
➜ cargo build --target x86_64-apple-darwin --release
```

As with iOS, link with SystemConfiguration.framework when building your macOS app.


## Android Build Instructions

To build for android devices you must first download the appropriate binary
distribution of standard library for valid target platform. This can be done using rustup:

```sh
➜  rustup target add armv7-linux-androideabi
# there are other targets that might be appropriate
➜  rustup show | grep android
```

To setup compilation for android you will also need to tell cargo 
about other compilation tools especialy linker.
Follow [NDK](https://developer.android.com/ndk) installation instructions.
Then configure cargo with linker and archiver:

```sh
# set valid paths!
➜  export ANDROID_HOME=/Users/$USER/Library/Android/sdk
➜  export NDK_HOME=$ANDROID_HOME/ndk/25.0.8775105

➜  mkdir ~/.cargo
➜  cat << EOF > ~/.cargo/config
[target.armv7-linux-androideabi]
ar = "$NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/llvm-ar"
linker = "$NDK_HOME/toolchains/llvm/prebuilt/darwin-x86_64/bin/armv7a-linux-androideabi24-clang++"
EOF

➜  cargo build --target armv7-linux-androideabi
```

This should work but currently ends with error due to [bug](https://github.com/rust-lang/rust/pull/85806):

```
  = note: ld: error: unable to find library -lgcc
          clang-14: error: linker command failed with exit code 1 (use -v to see invocation)
```

Ndk project has a workaround applied so should work out of the box:

```sh
➜  cargo install cargo-ndk
➜  rustup target add \
    aarch64-linux-android \
    armv7-linux-androideabi \
➜  cargo ndk \
    -t armeabi-v7a \
    -t arm64-v8a \
    -o ./target/jniLibs build --release
```

To use from java create 'OHttpNativeWrapper.java' with contents (the class name and the package are important because of JNI conventions)

```java
package org.platform;

class OHttpNativeWrapper {

    static {
        System.loadLibrary("apprelay");
    }

    private static native long encapsulateRequest(byte[] config, byte[] msg);

    private static native byte[] getEncapsulatedRequest(long ctx_ptr);

    private static native void dropRequestContext(long ctx_ptr);

    private static native byte[] decapsulateResponse(long ctx_ptr, byte[] encapsulated_response);
    
    public static native String lastErrorMessage();

    public static native void init();

    public static native void drop(long ctx_ptr);
}
```

And pass library using VM arguments:

```
# lib directory should contain libapprelay.so 
-Djava.library.path="lib/"
```

## Building size optimized binaries

To build binaries with a smaller disk footprint you can use the `release-space-optimized` profile:

```sh
# for iOS
➜  cargo build \
    --target aarch64-apple-ios \
    --no-default-features \
    --profile release-space-optimized

# for Android this will fail using ndk but the binaries will be located 
# in different directory. See https://github.com/bbqsrc/cargo-ndk/issues/73
➜  cargo ndk \
    -t armeabi-v7a \
    -t arm64-v8a \
    -o ./target/jniLibs build --profile release-space-optimized
```

For more background about the parameters set for this profile read [this repo](https://github.com/johnthagen/min-sized-rust).

## Logging in runtime

The library uses crate `env_logger` configured to log to stdout. To enable logging set environment variable:

```
RUST_LOG=debug
```

And in your application be sure to call function `initialize_logging` for C API or `init` for JNI.
Initialization function can be called only once.

