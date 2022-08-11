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

//
// # Safety
// This dereferences a raw pointer to `RequestContext` passed by user.
// Be sure that the context has not been yet freed and that you are using valid pointer
//
// <https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer>
uint8_t *request_context_message_ffi(struct RequestContext *context);

// Return the number of bytes that the encapsulated request takes
//
// # Safety
// This dereferences a raw pointer to `RequestContext` passed by user.
// Be sure that the context has not been yet freed and that you are using valid pointer
//
// <https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer>
size_t request_context_message_len_ffi(struct RequestContext *context);

// Return the pointer to first byte of decapsulated response
//
// # Safety
// This dereferences a raw pointer to `RequestContext` passed by user.
// Be sure that the context has not been yet freed and that you are using valid pointer
//
// <https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer>
uint8_t *response_context_message_ffi(struct ResponseContext *context);

// Return the number of bytes that the decapsulated response takes
//
// # Safety
// This dereferences a raw pointer to `RequestContext` passed by user.
// Be sure that the context has not been yet freed and that you are using valid pointer
//
// <https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer>
size_t response_context_message_len_ffi(struct ResponseContext *context);

// Encapsulates the provided `encoded_msg` using `encoded_config`
//
// This function will return `null` pointer if:
//     - creating the request context fails ie due to parsing errors of configuration
//     - encapsulation fails ie due to failed hpke encryption
//
// # Safety
// This dereferences a raw pointer to `RequestContext` passed by user.
// Be sure that the context has not been yet freed and that you are using valid pointer
//
// <https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer>
struct RequestContext *encapsulate_request_ffi(const uint8_t *encoded_config_ptr,
                                               size_t encoded_config_len,
                                               const uint8_t *encoded_msg_ptr,
                                               size_t encoded_msg_len);

// Decapsulates the provided `encapsulated_response` using `context`
//
// This function will return `null` pointer if:
//     - decapsulation fails ie due to failed hpke encryption
//
// # Safety
// This dereferences a raw pointer to `RequestContext` passed by user.
// Be sure that the context has not been yet freed and that you are using valid pointer
//
// <https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html#dereferencing-a-raw-pointer>
struct ResponseContext *decapsulate_response_ffi(struct RequestContext *context,
                                                 const uint8_t *encapsulated_response_ptr,
                                                 size_t encapsulated_response_len);

// Return the number of bytes in the last error message
// Does not include any trailing null terminators
int last_error_length(void);

// Write most recent error UTF-8 encoded message into a provided buffer
//
// If there are no recent errors then this returns `0`
// `-1` is returned if there is an error but something bad happened:
//     - provided `buffer` is to small
//     - or a provided `buffer` is a null pointer
// Otherewise the function returnes the number of bytes written to buffer
int last_error_message(char *buffer, int length);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
