// Copyright (c) 2022 Cloudflare, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

#![allow(clippy::unused_unit)]

use error_ffi::update_last_error;
use ohttp::hpke::{Aead, Kdf};
use ohttp::{ClientRequest, ClientResponse, KeyConfig, SymmetricSuite};
use std::any::Any;
use std::ptr::null_mut;
use std::{ptr, slice};

use std::panic::catch_unwind;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Failed to create request context")]
    RequestContextInitialization(#[source] ohttp::Error),
    #[error("Failed to encapsulate request")]
    EncapsulationFailed(#[source] ohttp::Error),
    #[error("Failed to decapsulate request")]
    DecapsulationFailed(#[source] ohttp::Error),

    #[error("Invalid argument `{0}` passed")]
    InvalidArgument(String),

    #[error("Panic unwinded at {0:?}")]
    SafePanic(Box<dyn Any + Send>),

    #[cfg(feature = "java")]
    #[error("Unexpected JNI issue")]
    JniProblem(#[source] jni::errors::Error),
}

#[cfg(feature = "java")]
pub mod android;

pub mod error_ffi;

pub struct RequestContext {
    encapsulated_request: Vec<u8>,
    response_context: ClientResponse,
}

#[macro_export]
macro_rules! null_safe_ptr {
    ($ptr:ident, $null_expr:expr, $deref:expr) => {
        if $ptr.is_null() {
            update_last_error(ClientError::InvalidArgument(format!(
                "Passed null pointer argument {}",
                stringify!(ident)
            )));
            return $null_expr;
        } else {
            $deref
        }
    };
}

macro_rules! catch_panics {
    ($possibly_panic:expr, $error_ret:expr) => {
        match catch_unwind(|| $possibly_panic) {
            Ok(ctx) => ctx,
            Err(err) => {
                let err = ClientError::SafePanic(err);
                update_last_error(err);
                $error_ret
            }
        }
    };
}

#[macro_export]
macro_rules! safe_unwrap {
    ($possibly_err:expr, $error_ret:expr, $err_context:expr) => {
        match $possibly_err {
            Ok(ret) => ret,
            Err(err) => {
                let err = $err_context(err);
                update_last_error(err);
                return $error_ret;
            }
        }
    };
}

/// Return a pointer to encapsulated request
///
/// # Safety
/// Dereferences a pointer to `RequestContext` passed by the caller.
/// Be sure that the context has not been yet freed and that you are using a valid pointer.
///
/// <https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer>
#[no_mangle]
pub unsafe extern "C" fn request_context_message_ffi(context: *mut RequestContext) -> *mut u8 {
    null_safe_ptr!(
        context,
        ptr::null_mut(),
        (*(*Box::into_raw(Box::new(context))))
            .encapsulated_request
            .as_mut_ptr() as *mut u8
    )
}

/// Return the size in bytes of the encapsulated request.
///
/// # Safety
/// Dereferences a pointer to `RequestContext` passed by the caller.
/// Be sure that the context has not been yet freed and that you are using a valid pointer.
///
/// <https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer>
#[no_mangle]
pub unsafe extern "C" fn request_context_message_len_ffi(
    context: *mut RequestContext,
) -> libc::size_t {
    null_safe_ptr!(
        context,
        0,
        (*(*Box::into_raw(Box::new(context))))
            .encapsulated_request
            .len()
    )
}

/// Frees up context memory. Call this ONLY if you did not call `decapsulate_response_ffi`.
///
/// Use cases:
/// - After `encapsulate_request_ffi` if you decide not to send the request
/// - If the HTTP request itself failed (network error, etc.) before receiving a response
///
/// Do NOT call this after `decapsulate_response_ffi` - that function always consumes
/// the context regardless of success or failure.
///
/// # Safety
/// Dereferences a pointer to `RequestContext` passed by the caller.
/// Be sure that the context has not been yet freed and that you are using a valid pointer.
///
/// <https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer>
#[no_mangle]
pub unsafe extern "C" fn request_context_message_drop_ffi(context: *mut RequestContext) {
    null_safe_ptr!(context, (), {
        let _context = Box::from_raw(context);
    })
}

pub struct ResponseContext {
    response: Vec<u8>,
}

/// Return a pointer to the decapsulated response.
///
/// # Safety
/// Dereferences a pointer to `RequestContext` passed by the caller.
/// Be sure that the context has not been yet freed and that you are using a valid pointer.
///
/// <https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer>
#[no_mangle]
pub unsafe extern "C" fn response_context_message_ffi(context: *mut ResponseContext) -> *mut u8 {
    null_safe_ptr!(
        context,
        ptr::null_mut(),
        (*(*Box::into_raw(Box::new(context)))).response.as_mut_ptr() as *mut u8
    )
}

/// Return size in bytes of the decapsulated response.
///
/// # Safety
/// Dereferences a pointer to `RequestContext` passed by the caller.
/// Be sure that the context has not been yet freed and that you are using a valid pointer.
///
/// <https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer>
#[no_mangle]
pub unsafe extern "C" fn response_context_message_len_ffi(
    context: *mut ResponseContext,
) -> libc::size_t {
    null_safe_ptr!(
        context,
        0,
        (*(*Box::into_raw(Box::new(context)))).response.len()
    )
}

/// Creates a request from the first supported config in `encoded_config_list`.
///
/// Gateways list their preferred config first (privacy-gateway-server-go lists X-Wing first).
/// `ClientRequest::from_encoded_config_list` takes the last supported config instead.
fn client_request_from_config_list(
    encoded_config_list: &[u8],
) -> Result<ClientRequest, ohttp::Error> {
    KeyConfig::decode_list(encoded_config_list)?
        .iter_mut()
        .filter(|config| has_supported_suite(config))
        .find_map(|config| ClientRequest::from_config(config).ok())
        .ok_or(ohttp::Error::Unsupported)
}

/// `KeyConfig::decode` drops unsupported suites but can keep a config with none left,
/// and `ClientRequest::from_config` panics on such a config.
fn has_supported_suite(config: &KeyConfig) -> bool {
    const KDFS: [Kdf; 3] = [Kdf::HkdfSha256, Kdf::HkdfSha384, Kdf::HkdfSha512];
    const AEADS: [Aead; 3] = [Aead::Aes128Gcm, Aead::Aes256Gcm, Aead::ChaCha20Poly1305];
    KDFS.iter().any(|&kdf| {
        AEADS
            .iter()
            .any(|&aead| config.select(SymmetricSuite::new(kdf, aead)).is_ok())
    })
}

/// Encapsulates the provided `encoded_msg` using the first supported config in
/// `encoded_config_list` and returns a context used for decapsulating the corresponding response.
///
/// This function will return a NULL pointer if:
/// - creating the request context fails due to input errors.
/// - encapsulation fails.
///
/// # Safety
/// Dereferences a pointer to `RequestContext` passed by the caller.
/// Be sure that the context has not been yet freed and that you are using a valid pointer.
///
/// <https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer>
#[no_mangle]
pub unsafe extern "C" fn encapsulate_request_ffi(
    encoded_config_list_ptr: *const u8,
    encoded_config_list_len: libc::size_t,
    encoded_msg_ptr: *const u8,
    encoded_msg_len: libc::size_t,
) -> *mut RequestContext {
    let encoded_config_list_ptr = null_safe_ptr!(
        encoded_config_list_ptr,
        ptr::null_mut(),
        encoded_config_list_ptr
    );
    let encoded_msg_ptr = null_safe_ptr!(encoded_msg_ptr, ptr::null_mut(), encoded_msg_ptr);

    let encoded_config_list: &[u8] = slice::from_raw_parts_mut(
        encoded_config_list_ptr as *mut u8,
        encoded_config_list_len as usize,
    );
    let encoded_msg: &[u8] =
        slice::from_raw_parts_mut(encoded_msg_ptr as *mut u8, encoded_msg_len as usize);

    catch_panics!(
        {
            let client = safe_unwrap!(
                { client_request_from_config_list(encoded_config_list) },
                ptr::null_mut(),
                ClientError::RequestContextInitialization
            );

            let (enc_request, client_response) = safe_unwrap!(
                client.encapsulate(encoded_msg),
                ptr::null_mut(),
                ClientError::EncapsulationFailed
            );

            let ctx = Box::new(RequestContext {
                encapsulated_request: enc_request,
                response_context: client_response,
            });
            Box::into_raw(ctx)
        },
        ptr::null_mut()
    )
}

/// Decapsulates the provided `encapsulated_response` using `context`.
///
/// This function **always consumes the context**, whether decapsulation succeeds or fails.
/// Do NOT call `request_context_message_drop_ffi` after calling this function.
///
/// Returns a pointer to the decapsulated response on success, or NULL on failure.
/// On failure, call `last_error_message_ffi` to get error details.
///
/// # Safety
/// Dereferences a pointer to `RequestContext` passed by the caller.
/// Be sure that the context has not been yet freed and that you are using a valid pointer.
///
/// <https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer>
#[no_mangle]
pub unsafe extern "C" fn decapsulate_response_ffi(
    context: *mut RequestContext,
    encapsulated_response_ptr: *const u8,
    encapsulated_response_len: libc::size_t,
) -> *mut ResponseContext {
    let context = null_safe_ptr!(context, null_mut(), Box::from_raw(context));

    let encapsulated_response_ptr = null_safe_ptr!(
        encapsulated_response_ptr,
        ptr::null_mut(),
        encapsulated_response_ptr
    );

    let encapsulated_response: &[u8] = slice::from_raw_parts_mut(
        encapsulated_response_ptr as *mut u8,
        encapsulated_response_len as usize,
    );

    catch_panics!(
        {
            let response = safe_unwrap!(
                context.response_context.decapsulate(encapsulated_response),
                ptr::null_mut(),
                ClientError::DecapsulationFailed
            );
            Box::into_raw(Box::new(ResponseContext { response }))
        },
        ptr::null_mut()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use ohttp::hpke::Kem;

    const XWING_KEY_ID: u8 = 1;
    const X25519_KEY_ID: u8 = 2;

    fn config(key_id: u8, kem: Kem) -> KeyConfig {
        let suites = vec![SymmetricSuite::new(Kdf::HkdfSha256, Aead::Aes128Gcm)];
        KeyConfig::new(key_id, kem, suites).unwrap()
    }

    /// Prefixes an encoded config with its length, as in an encoded config list.
    fn entry(encoded_config: &[u8]) -> Vec<u8> {
        [
            &(encoded_config.len() as u16).to_be_bytes()[..],
            encoded_config,
        ]
        .concat()
    }

    /// An entry with `X25519Kyber768Draft00`, which ohttp 0.8 does not support.
    fn unsupported_kem_entry() -> Vec<u8> {
        let suites = [0x00, 0x04, 0x00, 0x01, 0x00, 0x01];
        entry(&[&[3, 0x00, 0x30][..], &[0; 1216], &suites].concat())
    }

    /// An entry with a supported KEM and only HKDF-SHA384 + AES-256-GCM, which ohttp 0.8 does not support.
    fn unsupported_suite_entry(key_id: u8, kem: Kem) -> Vec<u8> {
        let mut encoded = config(key_id, kem).encode().unwrap();
        let suite_start = encoded.len() - 4;
        encoded[suite_start..].copy_from_slice(&[0x00, 0x02, 0x00, 0x02]);
        entry(&encoded)
    }

    /// Returns the key ID and the KEM ID from the header of the encapsulated request.
    fn selected_key(encoded_config_list: &[u8]) -> (u8, u16) {
        let client = client_request_from_config_list(encoded_config_list).unwrap();
        let (request, _) = client.encapsulate(b"request").unwrap();
        (request[0], u16::from_be_bytes([request[1], request[2]]))
    }

    #[test]
    fn uses_first_config() {
        let list = KeyConfig::encode_list(&[
            config(XWING_KEY_ID, Kem::XWing),
            config(X25519_KEY_ID, Kem::X25519Sha256),
        ])
        .unwrap();

        assert_eq!(selected_key(&list), (XWING_KEY_ID, u16::from(Kem::XWing)));
    }

    #[test]
    fn uses_first_config_in_reversed_list() {
        let list = KeyConfig::encode_list(&[
            config(X25519_KEY_ID, Kem::X25519Sha256),
            config(XWING_KEY_ID, Kem::XWing),
        ])
        .unwrap();

        assert_eq!(
            selected_key(&list),
            (X25519_KEY_ID, u16::from(Kem::X25519Sha256))
        );
    }

    #[test]
    fn uses_first_config_in_privacy_gateway_server_order() {
        let list = [
            entry(&config(XWING_KEY_ID, Kem::XWing).encode().unwrap()),
            unsupported_kem_entry(),
            entry(&config(X25519_KEY_ID, Kem::X25519Sha256).encode().unwrap()),
        ]
        .concat();

        assert_eq!(selected_key(&list), (XWING_KEY_ID, u16::from(Kem::XWing)));
    }

    #[test]
    fn skips_config_with_unsupported_kem() {
        let supported =
            KeyConfig::encode_list(&[config(X25519_KEY_ID, Kem::X25519Sha256)]).unwrap();
        let list = [unsupported_kem_entry(), supported].concat();

        assert_eq!(
            selected_key(&list),
            (X25519_KEY_ID, u16::from(Kem::X25519Sha256))
        );
    }

    #[test]
    fn skips_config_without_supported_suite() {
        let supported =
            KeyConfig::encode_list(&[config(X25519_KEY_ID, Kem::X25519Sha256)]).unwrap();
        let list = [unsupported_suite_entry(XWING_KEY_ID, Kem::XWing), supported].concat();

        assert_eq!(
            selected_key(&list),
            (X25519_KEY_ID, u16::from(Kem::X25519Sha256))
        );
    }

    #[test]
    fn fails_without_supported_config() {
        let list = [
            unsupported_kem_entry(),
            unsupported_suite_entry(XWING_KEY_ID, Kem::XWing),
        ]
        .concat();

        assert!(matches!(
            client_request_from_config_list(&list),
            Err(ohttp::Error::Unsupported)
        ));
    }
}
