use jni::JNIEnv;

use jni::objects::JClass;

use jni::sys::{jbyteArray, jlong, jstring};

use crate::error_ffi::update_last_error;
use crate::{ClientError, RequestContext};

/// Return most recent error as java `String`
///
/// If the are no recent errors than this returns `null` pointer
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

/// Encapsulates given request with provided configuration
///
/// Returns a pointer to encapsulation context.
/// If the provided values are null or function fails from other reason
/// the return value will be `-1`
#[no_mangle]
pub extern "system" fn Java_org_platform_OHttpNativeWrapper_encapsulateRequest(
    env: JNIEnv,
    _class: JClass,
    config: jbyteArray,
    msg: jbyteArray,
) -> jlong {
    if config.is_null() {
        update_last_error(ClientError::InvalidArgument("config".to_string()));
        return -1;
    }

    if msg.is_null() {
        update_last_error(ClientError::InvalidArgument("msg".to_string()));
        return -1;
    }

    // First, we have to get the byte[] out of java.
    let config = match env.convert_byte_array(config) {
        Ok(c) => c,
        Err(err) => {
            update_last_error(ClientError::JniProblem(err));
            return -1;
        }
    };

    let msg = match env.convert_byte_array(msg) {
        Ok(c) => c,
        Err(err) => {
            update_last_error(ClientError::JniProblem(err));
            return -1;
        }
    };

    unsafe {
        crate::encapsulate_request_ffi(config.as_ptr(), config.len(), msg.as_ptr(), msg.len())
            as jlong
    }
}

/// Accesses the encapsulation result for given context
///
/// Returns an array containing encapsulated request ready for ohttp.
///
/// If the jni failes to create array the return value will be `-1`
///
/// # Safety
/// This dereferences a raw pointer to `RequestContext` passed by user.
/// Be sure that the context has not been yet freed and that you are using valid pointer
///
/// <https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer>
#[no_mangle]
pub unsafe extern "system" fn Java_org_platform_OHttpNativeWrapper_getEncapsulatedRequest(
    env: JNIEnv,
    _class: JClass,
    context_ptr: jlong,
) -> jbyteArray {
    let context = &mut *(context_ptr as *mut RequestContext);
    match env.byte_array_from_slice(&context.encapsulated_request[..]) {
        Ok(req) => req,
        Err(err) => {
            update_last_error(ClientError::JniProblem(err));
            std::ptr::null_mut() as _
        }
    }
}

/// Frees up context memory. Be sure to call this in cases:
/// - after encapsulating the http request was not performed
/// - the response has not been returned or is not successfull
///
/// # Safety
/// This dereferences a raw pointer to `RequestContext` passed by user.
/// Be sure that the context has not been yet freed and that you are using valid pointer
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
/// Returns an array containing decapsulated response
///
/// If this funtion failes due to jni problems or decapsulation
/// it will return null pointer
///
/// # Safety
/// This dereferences a raw pointer to `RequestContext` passed by user.
/// Be sure that the context has not been yet freed and that you are using valid pointer
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
    let encapsulated_response = match env.convert_byte_array(encapsulated_response) {
        Ok(rsp) => rsp,
        Err(err) => {
            update_last_error(ClientError::JniProblem(err));
            return std::ptr::null_mut() as _;
        }
    };
    let response = match context.response_context.decapsulate(&encapsulated_response) {
        Ok(rsp) => rsp,
        Err(err) => {
            update_last_error(ClientError::DecapsulationFailed(err));
            return std::ptr::null_mut();
        }
    };
    match env.byte_array_from_slice(&response[..]) {
        Ok(rsp) => rsp,
        Err(err) => {
            update_last_error(ClientError::JniProblem(err));
            std::ptr::null_mut()
        }
    }
}
