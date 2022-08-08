use jni::JNIEnv;

use jni::objects::JClass;

use jni::sys::{jbyteArray, jlong};
use ohttp::ClientRequest;

use crate::RequestContext;

#[no_mangle]
pub extern "system" fn Java_org_platform_OHttpNativeWrapper_encapsulateRequest(
    env: JNIEnv,
    _class: JClass,
    config: jbyteArray,
    msg: jbyteArray,
) -> jlong {
    // First, we have to get the byte[] out of java.
    let config = env.convert_byte_array(config).unwrap();
    let msg = env.convert_byte_array(msg).unwrap();

    let client = ClientRequest::new(&config[..]).unwrap();
    let (enc_request, client_response) = client.encapsulate(&msg[..]).unwrap();
    Box::into_raw(Box::new(RequestContext {
        encapsulated_request: enc_request,
        response_context: client_response,
    })) as jlong
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_platform_OHttpNativeWrapper_getEncapsulatedRequest(
    env: JNIEnv,
    _class: JClass,
    context_ptr: jlong,
) -> jbyteArray {
    let context = &mut *(context_ptr as *mut RequestContext);
    env.byte_array_from_slice(&context.encapsulated_request[..])
        .unwrap()
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_platform_OHttpNativeWrapper_drop(
    _env: JNIEnv,
    _class: JClass,
    context_ptr: jlong,
) {
    let _context = Box::from_raw(context_ptr as *mut RequestContext);
}

#[no_mangle]
pub unsafe extern "system" fn Java_org_platform_OHttpNativeWrapper_decapsulateResponse(
    env: JNIEnv,
    _class: JClass,
    context_ptr: jlong,
    encapsulated_response: jbyteArray,
) -> jbyteArray {
    let context = Box::from_raw(context_ptr as *mut RequestContext);
    let encapsulated_response = env.convert_byte_array(encapsulated_response).unwrap();
    let response = context
        .response_context
        .decapsulate(&encapsulated_response)
        .unwrap();
    env.byte_array_from_slice(&response[..]).unwrap()
}
