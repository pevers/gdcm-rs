#include "gdcmImageReader.h"
#include "gdcmImage.h"

#include <string>

#include "wrapper.h"

using namespace std;

/**
 * Get the decoded pixel data from a DICOM file
 * 
 * @param i_buffer_ptr  pointer to the DICOM data
 * @param i_buffer_len  length of the DICOM data
 * 
 * @returns PixelData
 */
struct PixelData c_decode_dicom_file(
    char * i_buffer_ptr, 
    size_t i_buffer_len
) {
    struct PixelData outputStruct;
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
    outputStruct.size = length;
    outputStruct.status = 0;
    return outputStruct;
}

// struct PixelData c_decode_multi_frame_compressed(
//     char ** i_buffer_ptrs,
//     size_t * i_buffer_lens,
//     size_t i_buffer_len,
//     unsigned int dims[3],
//     unsigned int pi_type,
//     unsigned int ts_type,
//     unsigned short samples_per_pixel,
//     unsigned short bits_allocated,
//     unsigned short bits_stored,
//     unsigned short high_bit,
//     unsigned short pixel_representation
// ) {
//     // Create fragments
//     gdcm::SequenceOfFragments fragments = gdcm::SequenceOfFragments();
//     for (unsigned int i = 0; i < i_buffer_len; i++) {
//         gdcm::Fragment fragment = gdcm::Fragment();
//         fragment.SetByteValue(i_buffer_ptrs[0], gdcm::VL(i_buffer_lens[i]));
//         fragments.AddFragment(fragment);
//     }

//     // Create encapsulating DataElement
//     gdcm::DataElement data_element = gdcm::DataElement(gdcm::Tag(0x7fe0, 0x0010));
//     data_element.SetValue(fragments);

//     // TODO: Move this to a method
//     gdcm::Image image = gdcm::Image();
//     image.SetNumberOfDimensions(i_buffer_len == 1 ? 2 : 3);
//     image.SetDimensions(dims);
//     image.SetDataElement(data_element);
//     image.SetPhotometricInterpretation(gdcm::PhotometricInterpretation(gdcm::PhotometricInterpretation::PIType(pi_type)));
//     image.SetTransferSyntax(gdcm::TransferSyntax(gdcm::TransferSyntax::TSType(ts_type)));
//     image.SetPixelFormat(gdcm::PixelFormat(samples_per_pixel, bits_allocated, bits_stored, high_bit, pixel_representation));

//     // TODO: PLANAR CONFIGURATION (more than 1 for samples per pixel)

//     struct PixelData outputStruct;
//     image.GetBuffer(outputStruct.buffer);
//     outputStruct.size = image.GetBufferLength();
//     outputStruct.status = 0;
//     return outputStruct;
// }

struct PixelData c_decode_single_frame_compressed(
    char * i_buffer_ptr,
    size_t i_buffer_len,
    unsigned int width,
    unsigned int height,
    unsigned int pi_type,
    unsigned int ts_type,
    unsigned short samples_per_pixel,
    unsigned short bits_allocated,
    unsigned short bits_stored,
    unsigned short high_bit,
    unsigned short pixel_representation
) {
    // Create fragment
    // We need a SmartPointer because of a GDCM bug
    // https://sourceforge.net/p/gdcm/mailman/gdcm-developers/thread/CB8517FD.82C8%25mkazanov%40gmail.com/
    gdcm::SmartPointer<gdcm::SequenceOfFragments> fragments = new gdcm::SequenceOfFragments();
    gdcm::Fragment fragment = gdcm::Fragment();
    fragment.SetByteValue(i_buffer_ptr, gdcm::VL(i_buffer_len));
    fragments->AddFragment(fragment);

    // Create encapsulating DataElement
    gdcm::DataElement data_element = gdcm::DataElement(gdcm::Tag(0x7fe0, 0x0010));
    data_element.SetValue(*fragments);

    // TODO: Move this to a method
    gdcm::Image image = gdcm::Image();
    image.SetNumberOfDimensions(2);
    unsigned int dims[] = {width, height};
    image.SetDimensions(dims);
    image.SetDataElement(data_element);
    image.SetPhotometricInterpretation(gdcm::PhotometricInterpretation(gdcm::PhotometricInterpretation::PIType(pi_type)));
    image.SetTransferSyntax(gdcm::TransferSyntax(gdcm::TransferSyntax::TSType(ts_type)));
    image.SetPixelFormat(gdcm::PixelFormat(samples_per_pixel, bits_allocated, bits_stored, high_bit, pixel_representation));

    // TODO: PLANAR CONFIGURATION (more than 1 for samples per pixel)

    struct PixelData outputStruct;
    size_t length = image.GetBufferLength();
    outputStruct.buffer = (char*)malloc(length);
    if (!image.GetBuffer(outputStruct.buffer)) {
        outputStruct.status = 1;
        return outputStruct;
    }
    outputStruct.size = length;
    outputStruct.status = 0;
    return outputStruct;
}