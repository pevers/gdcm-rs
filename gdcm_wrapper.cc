#include "gdcmImageReader.h"
#include "gdcmImage.h"
#include "wrapper.h"

using namespace std;

struct PixelData c_decode_single_frame_compressed(
    char *i_buffer_ptr,
    size_t i_buffer_len,
    unsigned int width,
    unsigned int height,
    unsigned int pi_type,
    unsigned int ts_type,
    unsigned short samples_per_pixel,
    unsigned short bits_allocated,
    unsigned short bits_stored,
    unsigned short high_bit,
    unsigned short pixel_representation)
{
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

    struct PixelData outputStruct;
    size_t length = image.GetBufferLength();
    outputStruct.buffer = (char *)malloc(length);
    if (!image.GetBuffer(outputStruct.buffer))
    {
        outputStruct.status = 1;
        return outputStruct;
    }
    outputStruct.size = length;
    outputStruct.status = 0;
    return outputStruct;
}

struct PixelData c_decode_multi_frame_compressed(
    char **i_buffer_ptr,
    size_t *i_buffer_lens,
    size_t i_buffer_len,
    unsigned int dims[3],
    unsigned int pi_type,
    unsigned int ts_type,
    unsigned short samples_per_pixel,
    unsigned short bits_allocated,
    unsigned short bits_stored,
    unsigned short high_bit,
    unsigned short pixel_representation)
{
    // Create fragment
    // We need a SmartPointer because of a GDCM bug
    // https://sourceforge.net/p/gdcm/mailman/gdcm-developers/thread/CB8517FD.82C8%25mkazanov%40gmail.com/
    gdcm::SmartPointer<gdcm::SequenceOfFragments> fragments = new gdcm::SequenceOfFragments();
    for (size_t i_buffer_idx = 0; i_buffer_idx < i_buffer_len; ++i_buffer_idx)
    {
        gdcm::Fragment fragment = gdcm::Fragment();
        fragment.SetByteValue(i_buffer_ptr[i_buffer_idx], gdcm::VL(i_buffer_lens[i_buffer_idx]));
        fragments->AddFragment(fragment);
    }

    // Create encapsulating DataElement
    gdcm::DataElement data_element = gdcm::DataElement(gdcm::Tag(0x7fe0, 0x0010));
    data_element.SetValue(*fragments);

    // TODO: Move this to a method
    gdcm::Image image = gdcm::Image();
    image.SetNumberOfDimensions(3);
    image.SetDimensions(dims);
    image.SetDataElement(data_element);
    image.SetPhotometricInterpretation(gdcm::PhotometricInterpretation(gdcm::PhotometricInterpretation::PIType(pi_type)));
    image.SetTransferSyntax(gdcm::TransferSyntax(gdcm::TransferSyntax::TSType(ts_type)));
    image.SetPixelFormat(gdcm::PixelFormat(samples_per_pixel, bits_allocated, bits_stored, high_bit, pixel_representation));

    struct PixelData outputStruct;
    size_t length = image.GetBufferLength();
    outputStruct.buffer = (char *)malloc(length);
    if (!image.GetBuffer(outputStruct.buffer))
    {
        outputStruct.status = 1;
        return outputStruct;
    }
    outputStruct.size = length;
    outputStruct.status = 0;
    return outputStruct;
}