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

uint8_t *request_context_message_ffi(struct RequestContext *context);

size_t request_context_message_len_ffi(struct RequestContext *context);

uint8_t *response_context_message_ffi(struct ResponseContext *context);

size_t response_context_message_len_ffi(struct ResponseContext *context);

struct RequestContext *encapsulate_request_ffi(const uint8_t *encoded_config_ptr,
                                               size_t encoded_config_len,
                                               const uint8_t *encoded_msg_ptr,
                                               size_t encoded_msg_len);

struct ResponseContext *decapsulate_response_ffi(struct RequestContext *context,
                                                 const uint8_t *encapsulated_response_ptr,
                                                 size_t encapsulated_response_len);

#ifdef __cplusplus
} // extern "C"
#endif // __cplusplus
