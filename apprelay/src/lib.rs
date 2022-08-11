// Copyright (c) 2022 Cloudflare, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use error_ffi::update_last_error;
use ohttp::{ClientRequest, ClientResponse};
use std::{ptr, slice};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Failed to create request context")]
    RequestContextInitialization(ohttp::Error),
    #[error("Failed to encapsulate request")]
    EncapsulationFailed(ohttp::Error),
    #[error("Failed to decapsulate request")]
    DecapsulationFailed(ohttp::Error),

    #[error("Invalid argument `{0}` passed")]
    InvalidArgument(String),

    #[cfg(feature = "java")]
    #[error("Unexpected JNI issue")]
    JniProblem(jni::errors::Error),
}

#[cfg(feature = "java")]
mod android;

mod error_ffi;

pub struct RequestContext {
    encapsulated_request: Vec<u8>,
    response_context: ClientResponse,
}

// Return the pointer to first byte of encapsulated request
///
/// # Safety
/// This dereferences a raw pointer to `RequestContext` passed by user.
/// Be sure that the context has not been yet freed and that you are using valid pointer
///
/// <https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer>
#[no_mangle]
pub unsafe extern "C" fn request_context_message_ffi(context: Box<RequestContext>) -> *mut u8 {
    (*Box::into_raw(context)).encapsulated_request.as_mut_ptr() as *mut u8
}

/// Return the number of bytes that the encapsulated request takes
///
/// # Safety
/// This dereferences a raw pointer to `RequestContext` passed by user.
/// Be sure that the context has not been yet freed and that you are using valid pointer
///
/// <https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer>
#[no_mangle]
pub unsafe extern "C" fn request_context_message_len_ffi(
    context: Box<RequestContext>,
) -> libc::size_t {
    (*Box::into_raw(context)).encapsulated_request.len()
}

pub struct ResponseContext {
    response: Vec<u8>,
}

/// Return the pointer to first byte of decapsulated response
///
/// # Safety
/// This dereferences a raw pointer to `RequestContext` passed by user.
/// Be sure that the context has not been yet freed and that you are using valid pointer
///
/// <https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer>
#[no_mangle]
pub unsafe extern "C" fn response_context_message_ffi(context: Box<ResponseContext>) -> *mut u8 {
    (*Box::into_raw(context)).response.as_mut_ptr() as *mut u8
}

/// Return the number of bytes that the decapsulated response takes
///
/// # Safety
/// This dereferences a raw pointer to `RequestContext` passed by user.
/// Be sure that the context has not been yet freed and that you are using valid pointer
///
/// <https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer>
#[no_mangle]
pub unsafe extern "C" fn response_context_message_len_ffi(
    context: Box<ResponseContext>,
) -> libc::size_t {
    (*Box::into_raw(context)).response.len()
}

/// Encapsulates the provided `encoded_msg` using `encoded_config`
///
/// This function will return `null` pointer if:
///     - creating the request context fails ie due to parsing errors of configuration
///     - encapsulation fails ie due to failed hpke encryption
///
/// # Safety
/// This dereferences a raw pointer to `RequestContext` passed by user.
/// Be sure that the context has not been yet freed and that you are using valid pointer
///
/// <https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer>
#[no_mangle]
pub unsafe extern "C" fn encapsulate_request_ffi(
    encoded_config_ptr: *const u8,
    encoded_config_len: libc::size_t,
    encoded_msg_ptr: *const u8,
    encoded_msg_len: libc::size_t,
) -> *mut RequestContext {
    let encoded_config: &[u8] =
        slice::from_raw_parts_mut(encoded_config_ptr as *mut u8, encoded_config_len as usize);
    let encoded_msg: &[u8] =
        slice::from_raw_parts_mut(encoded_msg_ptr as *mut u8, encoded_msg_len as usize);

    let client = match ClientRequest::new(encoded_config) {
        Ok(c) => c,
        Err(err) => {
            let err = ClientError::RequestContextInitialization(err);
            update_last_error(err);
            return ptr::null_mut();
        }
    };

    let (enc_request, client_response) = match client.encapsulate(encoded_msg) {
        Ok(encapsulated) => encapsulated,
        Err(err) => {
            let err = ClientError::EncapsulationFailed(err);
            update_last_error(err);
            return ptr::null_mut();
        }
    };

    let ctx = Box::new(RequestContext {
        encapsulated_request: enc_request,
        response_context: client_response,
    });

    Box::into_raw(ctx)
}

/// Decapsulates the provided `encapsulated_response` using `context`
///
/// This function will return `null` pointer if:
///     - decapsulation fails ie due to failed hpke encryption
///
/// # Safety
/// This dereferences a raw pointer to `RequestContext` passed by user.
/// Be sure that the context has not been yet freed and that you are using valid pointer
///
/// <https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer>
#[no_mangle]
pub unsafe extern "C" fn decapsulate_response_ffi(
    context: Box<RequestContext>,
    encapsulated_response_ptr: *const u8,
    encapsulated_response_len: libc::size_t,
) -> *mut ResponseContext {
    let encapsulated_response: &[u8] = slice::from_raw_parts_mut(
        encapsulated_response_ptr as *mut u8,
        encapsulated_response_len as usize,
    );
    let response = match context.response_context.decapsulate(encapsulated_response) {
        Ok(response) => response,
        Err(err) => {
            let err = ClientError::DecapsulationFailed(err);
            update_last_error(err);
            return ptr::null_mut();
        }
    };
    Box::into_raw(Box::new(ResponseContext { response }))
}
