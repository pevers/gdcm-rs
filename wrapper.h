struct OutputStruct {
    unsigned int status;
    size_t size;
};

struct PixelDataStruct {
    char *buffer;
    unsigned int status;
    size_t size;
};

#ifdef __cplusplus
extern "C" {
#endif

#ifdef _WIN32
#  ifdef MODULE_API_EXPORTS
#    define MODULE_API __declspec(dllexport)
#  else
#    define MODULE_API __declspec(dllimport)
#  endif
#else
#  define MODULE_API
#endif

MODULE_API OutputStruct c_convert(
    char *,     // i_buffer_ptr
    size_t,     // i_buffer_len
    size_t,     // max_size
    int,        // transfer_syntax
    char,       // is_lossy
    int,        // quality1
    int,        // quality2
    int,        // quality3
    char,       // irreversible
    int,        // allow_error
    char *,     // implementation_version_name
    char *      // source_application_entity_title
);

MODULE_API PixelDataStruct c_get_pixel_data(
    char *,     // i_buffer_ptr
    size_t     // i_buffer_len
);

#ifdef __cplusplus
}
#endif