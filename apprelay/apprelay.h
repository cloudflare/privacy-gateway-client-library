#include <stdarg.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct RequestContext RequestContext;

typedef struct ResponseContext ResponseContext;

#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

// Return a pointer to encapsulated request
//
// # Safety
// Dereferences a pointer to `RequestContext` passed by the caller.
// Be sure that the context has not been yet freed and that you are using a valid pointer.
//
// <https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer>
uint8_t *request_context_message_ffi(struct RequestContext *context);

// Return the size in bytes of the encapsulated request.
//
// # Safety
// Dereferences a pointer to `RequestContext` passed by the caller.
// Be sure that the context has not been yet freed and that you are using a valid pointer.
//
// <https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer>
size_t request_context_message_len_ffi(struct RequestContext *context);

// Frees up context memory. Be sure to call this in cases:
// - after encapsulating the HTTP request was not performed
// - the response has not been returned or is not successful
//
// # Safety
// Dereferences a pointer to `RequestContext` passed by the caller.
// Be sure that the context has not been yet freed and that you are using a valid pointer.
//
// <https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer>
void request_context_message_drop_ffi(struct RequestContext *context);

// Return a pointer to the decapsulated response.
//
// # Safety
// Dereferences a pointer to `RequestContext` passed by the caller.
// Be sure that the context has not been yet freed and that you are using a valid pointer.
//
// <https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer>
uint8_t *response_context_message_ffi(struct ResponseContext *context);

// Return size in bytes of the decapsulated response.
//
// # Safety
// Dereferences a pointer to `RequestContext` passed by the caller.
// Be sure that the context has not been yet freed and that you are using a valid pointer.
//
// <https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer>
size_t response_context_message_len_ffi(struct ResponseContext *context);

// Encapsulates the provided `encoded_msg` using `encoded_config_list` and returns
// a context used for decapsulating the corresponding response.
//
// This function will return a NULL pointer if:
// - creating the request context fails due to input errors.
// - encapsulation fails.
//
// # Safety
// Dereferences a pointer to `RequestContext` passed by the caller.
// Be sure that the context has not been yet freed and that you are using a valid pointer.
//
// <https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer>
struct RequestContext *encapsulate_request_ffi(const uint8_t *encoded_config_list_ptr,
                                               size_t encoded_config_list_len,
                                               const uint8_t *encoded_msg_ptr,
                                               size_t encoded_msg_len);

// Decapsulates the provided `encapsulated_response` using `context`.
//
// This function will return a NULL pointer if decapsulation fails.
//
// # Safety
// Dereferences a pointer to `RequestContext` passed by the caller.
// Be sure that the context has not been yet freed and that you are using a valid pointer.
//
// <https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer>
struct ResponseContext *decapsulate_response_ffi(struct RequestContext *context,
                                                 const uint8_t *encapsulated_response_ptr,
                                                 size_t encapsulated_response_len);

void initialize_logging(void);

// Return the number of bytes in the last error message.
// Does not include any trailing null terminators.
int last_error_length(void);

// Write the most recent error UTF-8 encoded message into a provided buffer
//
// If there are no recent errors then this returns 0. -1 is returned if there is an error but something bad happened:
// - provided `buffer` is too small
// - or a provided `buffer` is a null pointer
//
// Otherwise the function returns the number of bytes written to the buffer.
//
// # Safety
// The invariants are described here [`from_raw_parts_mut`](std::slice::from_raw_parts_mut#safety)
int last_error_message(char *buffer,
                       int length);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
