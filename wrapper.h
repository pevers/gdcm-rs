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

    struct PixelData
    {
        char *buffer;
        unsigned int status;
        size_t size;
    };

    MODULE_API PixelData c_decode_frames(
        char **,         // i_buffer_ptrs
        size_t *,        // i_buffer_lens
        size_t,          // i_buffer_len
        unsigned int[3], // dims
        unsigned int,    // pi_type
        unsigned int,    // ts_type
        unsigned short,  // samples_per_pixel
        unsigned short,  // bits_allocated
        unsigned short,  // bits_stored
        unsigned short,  // high_bit
        unsigned short   // pixel_representation
    );
#ifdef __cplusplus
}
#endif