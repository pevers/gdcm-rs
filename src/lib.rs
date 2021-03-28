use libc::{c_uchar, c_uint, size_t};
use std::slice;
use strum_macros::EnumString;

/// PhotometricInterpretation Type for GDCM
#[derive(Debug, PartialEq, EnumString)]
#[allow(non_camel_case_types)]
pub enum GDCMPhotometricInterpretation {
    UNKNOWN = 0,
    MONOCHROME1,
    MONOCHROME2,
    PALETTE_COLOR,
    RGB,
    HSV,
    ARGB, // retired
    CMYK,
    YBR_FULL,
    YBR_FULL_422,
    YBR_PARTIAL_422,
    YBR_PARTIAL_420,
    YBR_ICT,
    YBR_RCT,
    PI_END,
}

pub type InvalidGDCMPI = strum::ParseError;

/// GDCM TransferSyntax encodings for GDCM
#[derive(Debug, PartialEq, EnumString)]
#[allow(non_camel_case_types)]
pub enum GDCMTransferSyntax {
    #[strum(serialize = "1.2.840.10008.1.2")]
    ImplicitVRLittleEndian = 0,
    ImplicitVRBigEndianPrivateGE, // Unknown
    #[strum(serialize = "1.2.840.10008.1.2.1")]
    ExplicitVRLittleEndian,
    #[strum(serialize = "1.2.840.10008.1.2.1.99")]
    DeflatedExplicitVRLittleEndian,
    #[strum(serialize = "1.2.840.10008.1.2.2")]
    ExplicitVRBigEndian,
    #[strum(serialize = "1.2.840.10008.1.2.4.50")]
    JPEGBaselineProcess1,
    #[strum(serialize = "1.2.840.10008.1.2.4.51")]
    JPEGExtendedProcess2_4,
    #[strum(serialize = "1.2.840.10008.1.2.4.52")]
    JPEGExtendedProcess3_5,
    #[strum(serialize = "1.2.840.10008.1.2.4.53")]
    JPEGSpectralSelectionProcess6_8,
    #[strum(serialize = "1.2.840.10008.1.2.4.55")]
    JPEGFullProgressionProcess10_12,
    #[strum(serialize = "1.2.840.10008.1.2.4.57")]
    JPEGLosslessProcess14,
    #[strum(serialize = "1.2.840.10008.1.2.4.70")]
    JPEGLosslessProcess14_1,
    #[strum(serialize = "1.2.840.10008.1.2.4.80")]
    JPEGLSLossless,
    #[strum(serialize = "1.2.840.10008.1.2.4.81")]
    JPEGLSNearLossless,
    #[strum(serialize = "1.2.840.10008.1.2.4.90")]
    JPEG2000Lossless,
    #[strum(serialize = "1.2.840.10008.1.2.4.91")]
    JPEG2000,
    #[strum(serialize = "1.2.840.10008.1.2.4.92")]
    JPEG2000Part2Lossless,
    #[strum(serialize = "1.2.840.10008.1.2.4.93")]
    JPEG2000Part2,
    #[strum(serialize = "1.2.840.10008.1.2.5")]
    RLELossless,
    #[strum(serialize = "1.2.840.10008.1.2.4.100")]
    MPEG2MainProfile,
    ImplicitVRBigEndianACRNEMA, // Unkown
    WeirdPapryus,               // Unknown
    CT_private_ELE,             // Unknown
    #[strum(serialize = "1.2.840.10008.1.2.4.95")]
    JPIPReferenced,
    #[strum(serialize = "1.2.840.10008.1.2.4.101")]
    MPEG2MainProfileHighLevel,
    #[strum(serialize = "1.2.840.10008.1.2.4.102")]
    MPEG4AVCH264HighProfileLevel4_1,
    #[strum(serialize = "1.2.840.10008.1.2.4.103")]
    MPEG4AVCH264BDcompatibleHighProfileLevel4_1,
    TS_END,
}

pub type InvalidGDCMTS = strum::ParseError;

/// TransferSyntax Type
pub enum TSType {}

#[repr(C)]
struct pixel_data {
    pixel_data: *const c_uchar,
    status: c_uint,
    size: size_t,
}

extern "C" {
    // fn c_decode_multi_frame_compressed(
    //     i_buffer_ptr: *const *const c_uchar,
    //     i_buffer_lens: *const size_t,
    //     i_buffer_len: size_t,
    //     dims: (i32, i32, i32),
    //     pi_type: u32,
    //     ts_type: u32,
    //     samples_per_pixel: u16,
    //     bits_allocated: u16,
    //     bits_stored: u16,
    //     high_bit: u16,
    //     pixel_representation: u16,
    // ) -> pixel_data;

    fn c_decode_single_frame_compressed(
        i_buffer_ptr: *const c_uchar,
        i_buffer_len: size_t,
        width: u32,
        height: u32,
        pi_type: u32,
        ts_type: u32,
        samples_per_pixel: u16,
        bits_allocated: u16,
        bits_stored: u16,
        high_bit: u16,
        pixel_representation: u16,
    ) -> pixel_data;
}

/// Takes an in-memory dicom-rs PixelData and decodes it
pub fn decode_single_frame_compressed(
    i_buffer: &Vec<u8>,
    width: u32,
    height: u32,
    pi_type: GDCMPhotometricInterpretation,
    ts_type: GDCMTransferSyntax,
    samples_per_pixel: u16,
    bits_allocated: u16,
    bits_stored: u16,
    high_bit: u16,
    pixel_representation: u16,
) -> Result<Box<[u8]>, &'static str> {
    let ret = unsafe {
        c_decode_single_frame_compressed(
            i_buffer.as_ptr(),
            i_buffer.len(),
            width,
            height,
            pi_type as u32,
            ts_type as u32,
            samples_per_pixel,
            bits_allocated,
            bits_stored,
            high_bit,
            pixel_representation,
        )
    };
    match ret.status {
        0 => unsafe {
            let slice = slice::from_raw_parts_mut(ret.pixel_data as *mut _, ret.size);
            return Ok(Box::from_raw(slice));
        },
        _ => return Err("Unknown error."),
    }
}
