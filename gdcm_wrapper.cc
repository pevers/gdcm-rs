#include "gdcmImageReader.h"
#include "gdcmImage.h"

#include <string>

#include "wrapper.h"

using namespace std;

/**
 * Get the decoded pixel data from a DICOM
 * 
 * @param i_buffer_ptr  pointer to the DICOM data
 * @param i_buffer_len  length of the DICOM data
 * 
 * @returns PixelDataStruct
 */
struct PixelDataStruct c_get_pixel_data(
    char * i_buffer_ptr, 
    size_t i_buffer_len
) {
    struct PixelDataStruct outputStruct;
    string inputString(i_buffer_ptr, i_buffer_len);
    istringstream dicomInput(inputString);

    gdcm::PixmapReader reader;
    reader.SetStream(dicomInput);

    // can't read stream
    if(!reader.Read()) {
        outputStruct.status = 2;
        return outputStruct;
    }

    gdcm::Pixmap &image = reader.GetPixmap();
    size_t length = image.GetBufferLength();

    outputStruct.buffer = (char*)malloc(length);
    if (!image.GetBuffer(outputStruct.buffer)) {
        outputStruct.status = 1;
        return outputStruct;
    }
    outputStruct.size = image.GetBufferLength();
    outputStruct.status = 0;
    return outputStruct;
}