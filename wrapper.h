#ifdef __cplusplus
extern "C"
{
#endif

#ifdef _WIN32
#ifdef MODULE_API_EXPORTS
#define MODULE_API __declspec(dllexport)
#else
#define MODULE_API __declspec(dllimport)
#endif
#else
#define MODULE_API
#endif

    struct PixelDataInternal
    {
        char *buffer;
        u_int32_t status;
        size_t size;
    };

    MODULE_API PixelDataInternal c_decode_frames(
        char **,      // i_buffer_ptrs
        size_t *,     // i_buffer_lens
        size_t,       // i_buffer_len
        u_int32_t[3], // dims
        u_int32_t,    // pi_type
        u_int32_t,    // ts_type
        u_int16_t,    // samples_per_pixel
        u_int16_t,    // bits_allocated
        u_int16_t,    // bits_stored
        u_int16_t,    // high_bit
        u_int16_t     // pixel_representation
    );

    MODULE_API void c_free_buffer(char *);
#ifdef __cplusplus
}
#endif