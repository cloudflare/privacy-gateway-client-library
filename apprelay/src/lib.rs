// Copyright (c) 2022 Cloudflare, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

#![allow(clippy::unused_unit)]

use error_ffi::update_last_error;
use ohttp::{ClientRequest, ClientResponse};
use std::any::Any;
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

/// Frees up context memory. Be sure to call this in cases:
/// - after encapsulating the HTTP request was not performed
/// - the response has not been returned or is not successful
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

/// Encapsulates the provided `encoded_msg` using `encoded_config` and returns
/// a context used for decapsulating the corresponding response.
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
    encoded_config_ptr: *const u8,
    encoded_config_len: libc::size_t,
    encoded_msg_ptr: *const u8,
    encoded_msg_len: libc::size_t,
) -> *mut RequestContext {
    let encoded_config_ptr =
        null_safe_ptr!(encoded_config_ptr, ptr::null_mut(), encoded_config_ptr);
    let encoded_msg_ptr = null_safe_ptr!(encoded_msg_ptr, ptr::null_mut(), encoded_msg_ptr);

    let encoded_config: &[u8] =
        slice::from_raw_parts_mut(encoded_config_ptr as *mut u8, encoded_config_len as usize);
    let encoded_msg: &[u8] =
        slice::from_raw_parts_mut(encoded_msg_ptr as *mut u8, encoded_msg_len as usize);

    catch_panics!(
        {
            let client = safe_unwrap!(
                { ClientRequest::new(encoded_config) },
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
/// This function will return a NULL pointer if decapsulation fails.
///
/// # Safety
/// Dereferences a pointer to `RequestContext` passed by the caller.
/// Be sure that the context has not been yet freed and that you are using a valid pointer.
///
/// <https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer>
#[no_mangle]
pub unsafe extern "C" fn decapsulate_response_ffi(
    context: Box<RequestContext>,
    encapsulated_response_ptr: *const u8,
    encapsulated_response_len: libc::size_t,
) -> *mut ResponseContext {
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
