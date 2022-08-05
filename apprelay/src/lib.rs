// Copyright (c) 2022 Cloudflare, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

use ohttp::{ClientRequest, ClientResponse};
use std::slice;

 #[cfg(feature = "java")]
mod android;

pub struct RequestContext {
    encapsulated_request: Vec<u8>,
    response_context: ClientResponse,
}

#[no_mangle]
pub unsafe extern "C" fn request_context_message_ffi(context: Box<RequestContext>) -> *mut u8 {
    (*Box::into_raw(context)).encapsulated_request.as_mut_ptr() as *mut u8
}

#[no_mangle]
pub unsafe extern "C" fn request_context_message_len_ffi(
    context: Box<RequestContext>,
) -> libc::size_t {
    (*Box::into_raw(context)).encapsulated_request.len()
}

pub struct ResponseContext {
    response: Vec<u8>,
}

#[no_mangle]
pub unsafe extern "C" fn response_context_message_ffi(context: Box<ResponseContext>) -> *mut u8 {
    (*Box::into_raw(context)).response.as_mut_ptr() as *mut u8
}

#[no_mangle]
pub unsafe extern "C" fn response_context_message_len_ffi(
    context: Box<ResponseContext>,
) -> libc::size_t {
    (*Box::into_raw(context)).response.len()
}

#[no_mangle]
pub unsafe extern "C" fn encapsulate_request_ffi(
    encoded_config_ptr: *const u8,
    encoded_config_len: libc::size_t,
    encoded_msg_ptr: *const u8,
    encoded_msg_len: libc::size_t,
) -> Box<RequestContext> {
    let encoded_config: &[u8] =
        slice::from_raw_parts_mut(encoded_config_ptr as *mut u8, encoded_config_len as usize);
    let encoded_msg: &[u8] =
        slice::from_raw_parts_mut(encoded_msg_ptr as *mut u8, encoded_msg_len as usize);
    let client = ClientRequest::new(encoded_config).unwrap();
    let (enc_request, client_response) = client.encapsulate(encoded_msg).unwrap();
    Box::new(RequestContext {
        encapsulated_request: enc_request,
        response_context: client_response,
    })
}

#[no_mangle]
pub unsafe extern "C" fn decapsulate_response_ffi(
    context: Box<RequestContext>,
    encapsulated_response_ptr: *const u8,
    encapsulated_response_len: libc::size_t,
) -> Box<ResponseContext> {
    let encapsulated_response: &[u8] = slice::from_raw_parts_mut(
        encapsulated_response_ptr as *mut u8,
        encapsulated_response_len as usize,
    );
    let response = context
        .response_context
        .decapsulate(encapsulated_response)
        .unwrap();
    Box::new(ResponseContext { response })
}
