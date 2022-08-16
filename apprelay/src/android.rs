use std::ptr::null_mut;

use jni::JNIEnv;

use jni::objects::JClass;

use jni::sys::{jbyteArray, jlong, jstring};

use crate::error_ffi::update_last_error;
use crate::{null_safe_ptr, safe_unwrap, ClientError, RequestContext};

/// Return most recent error as a Java `String`.
///
/// If the are no recent errors than this returns a NULL pointer.
#[no_mangle]
pub extern "system" fn Java_org_platform_OHttpNativeWrapper_lastErrorMessage(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    let err = crate::error_ffi::take_last_error();
    match err.map(|e| env.new_string(e.to_string())) {
        Some(Ok(jstr)) => jstr.into_inner(),
        _ => std::ptr::null_mut() as _,
    }
}

/// Initialize logging
#[no_mangle]
pub extern "system" fn Java_org_platform_OHttpNativeWrapper_init(_env: JNIEnv, _class: JClass) {
    crate::error_ffi::initialize_logging();
}

/// Encapsulates a request using the provided configuration.
///
/// Returns a pointer to encapsulation context, and returns -1 upon failure.
#[no_mangle]
pub extern "system" fn Java_org_platform_OHttpNativeWrapper_encapsulateRequest(
    env: JNIEnv,
    _class: JClass,
    config: jbyteArray,
    msg: jbyteArray,
) -> jlong {
    // check for null references passed
    null_safe_ptr!(config, -1, ());
    null_safe_ptr!(msg, -1, ());

    // First, we have to get the byte[] out of java.
    let config = crate::safe_unwrap!(env.convert_byte_array(config), -1, ClientError::JniProblem);
    let msg = crate::safe_unwrap!(env.convert_byte_array(msg), -1, ClientError::JniProblem);

    unsafe {
        let encapsulated =
            crate::encapsulate_request_ffi(config.as_ptr(), config.len(), msg.as_ptr(), msg.len());
        if encapsulated.is_null() {
            -1
        } else {
            encapsulated as jlong
        }
    }
}

/// Accesses the encapsulation result for given context.
///
/// Returns an array containing an encapsulation request,
/// or -1 upon failure.
///
/// # Safety
/// Dereferences a pointer to `RequestContext` passed by the caller.
/// Be sure that the context has not been yet freed and that you are using a valid pointer.
///
/// <https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer>
#[no_mangle]
pub unsafe extern "system" fn Java_org_platform_OHttpNativeWrapper_getEncapsulatedRequest(
    env: JNIEnv,
    _class: JClass,
    context_ptr: jlong,
) -> jbyteArray {
    let context = &mut *(context_ptr as *mut RequestContext);
    safe_unwrap!(
        env.byte_array_from_slice(&context.encapsulated_request[..]),
        null_mut(),
        ClientError::JniProblem
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
pub unsafe extern "system" fn Java_org_platform_OHttpNativeWrapper_drop(
    _env: JNIEnv,
    _class: JClass,
    context_ptr: jlong,
) {
    let _context = Box::from_raw(context_ptr as *mut RequestContext);
}

/// Decapsulates the provided response `encapsulated_response` using
/// requests config obtain by dereferencing `context_ptr`
///
/// Returns an array containing the decapsulated response.
///
/// If this function fails due JNI problems or decapsulation it returns a NULL pointer.
///
/// # Safety
/// Dereferences a pointer to `RequestContext` passed by the caller.
/// Be sure that the context has not been yet freed and that you are using a valid pointer.
///
/// <https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer>
#[no_mangle]
pub unsafe extern "system" fn Java_org_platform_OHttpNativeWrapper_decapsulateResponse(
    env: JNIEnv,
    _class: JClass,
    context_ptr: jlong,
    encapsulated_response: jbyteArray,
) -> jbyteArray {
    let context = Box::from_raw(context_ptr as *mut RequestContext);
    let encapsulated_response = crate::safe_unwrap!(
        env.convert_byte_array(encapsulated_response),
        null_mut(),
        ClientError::JniProblem
    );
    let response = safe_unwrap!(
        context.response_context.decapsulate(&encapsulated_response),
        null_mut(),
        ClientError::DecapsulationFailed
    );
    safe_unwrap!(
        env.byte_array_from_slice(&response[..]),
        null_mut(),
        ClientError::JniProblem
    )
}
