// Copyright (c) 2022 Cloudflare, Inc. All rights reserved.
// SPDX-License-Identifier: BSD-3-Clause

#[allow(dead_code)]

use std::slice;
use ohttp::{ClientRequest, ClientResponse};

pub struct RequestContext {
    encapsulated_request: Vec<u8>,
    response_context: ClientResponse,
}

#[no_mangle]
pub unsafe extern "C" fn request_context_message_ffi(context: Box<RequestContext>) -> *mut libc::uint8_t {
    (*Box::into_raw(context)).encapsulated_request.as_mut_ptr() as *mut libc::uint8_t
}

#[no_mangle]
pub unsafe extern "C" fn request_context_message_len_ffi(context: Box<RequestContext>) -> libc::size_t {
    (*Box::into_raw(context)).encapsulated_request.len()
}

pub struct ResponseContext {
    response: Vec<u8>,
}

#[no_mangle]
pub unsafe extern "C" fn response_context_message_ffi(context: Box<ResponseContext>) -> *mut libc::uint8_t {
    (*Box::into_raw(context)).response.as_mut_ptr() as *mut libc::uint8_t
}

#[no_mangle]
pub unsafe extern "C" fn response_context_message_len_ffi(context: Box<ResponseContext>) -> libc::size_t {
    (*Box::into_raw(context)).response.len()
}

#[no_mangle]
pub unsafe extern "C" fn encapsulate_request_ffi(
    encoded_config_ptr: *const libc::uint8_t,
    encoded_config_len: libc::size_t,
    encoded_msg_ptr: *const libc::uint8_t,
    encoded_msg_len: libc::size_t,
) -> Box<RequestContext> {
    let encoded_config: &[u8] = slice::from_raw_parts_mut(encoded_config_ptr as *mut u8, encoded_config_len as usize);
    let encoded_msg: &[u8] = slice::from_raw_parts_mut(encoded_msg_ptr as *mut u8, encoded_msg_len as usize);
    let client = ClientRequest::new(encoded_config).unwrap();
    let (enc_request, client_response) = client.encapsulate(encoded_msg).unwrap();
    Box::new(RequestContext{
        encapsulated_request: enc_request,
        response_context: client_response,
    })
}

#[no_mangle]
pub unsafe extern "C" fn decapsulate_response_ffi(
    context: Box<RequestContext>,
    encapsulated_response_ptr: *const libc::uint8_t,
    encapsulated_response_len: libc::size_t,
) -> Box<ResponseContext> {
    let encapsulated_response: &[u8] = slice::from_raw_parts_mut(encapsulated_response_ptr as *mut u8, encapsulated_response_len as usize);
    let response = (*context).response_context.decapsulate(encapsulated_response).unwrap();
    Box::new(ResponseContext{
        response: response,
    })
}
