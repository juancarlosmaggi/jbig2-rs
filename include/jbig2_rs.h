#ifndef JBIG2_RS_H
#define JBIG2_RS_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct Jbig2FfiDecodeOptions {
  uint64_t max_input_bytes;
  uint64_t max_decoded_pixels;
  uint64_t max_page_count;
  uint64_t max_segment_count;
  uint64_t max_symbol_dictionary_bytes;
  uint64_t max_intermediate_bitmap_bytes;
  size_t page_index;
  uint8_t collect_profile;
} Jbig2FfiDecodeOptions;

typedef struct Jbig2FfiResult {
  uint32_t code;
  uint32_t width;
  uint32_t height;
  size_t stride;
  size_t page_index;
  uint32_t page_id;
  uint8_t *data;
  size_t data_len;
  char *error_message;
  char *profile_report;
} Jbig2FfiResult;

Jbig2FfiDecodeOptions jbig2_ffi_decode_options_default(void);

/*
 * Decode one JBIG2 page.
 *
 * Input buffers are borrowed for the duration of the call. On success,
 * result.data points to packed 1bpp bytes, 8 pixels per byte, MSB-first, with
 * stride = (width + 7) / 8. A set bit means black/foreground.
 *
 * All non-null pointers in Jbig2FfiResult are owned by Rust and must be freed
 * with jbig2_ffi_result_free.
 */
Jbig2FfiResult jbig2_ffi_decode_page(const uint8_t *page_ptr,
                                     size_t page_len,
                                     const uint8_t *global_ptr,
                                     size_t global_len,
                                     const Jbig2FfiDecodeOptions *options);

void jbig2_ffi_result_free(Jbig2FfiResult result);

const char *jbig2_ffi_error_code_name(uint32_t code);

#ifdef __cplusplus
}
#endif

#endif
