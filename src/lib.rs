use libc::{c_uchar, size_t};
use snafu::{OptionExt, Snafu};
use std::{ptr::NonNull, slice};
use strum_macros::EnumString;

#[derive(Debug, Snafu)]
pub struct Error(InnerError);

#[derive(Debug, Snafu)]
enum InnerError {
    #[snafu(display("GDCM decoding error (status code {})", status))]
    GdcmDecodingError { status: u32 },

    #[snafu(display("Invalid pointer provided to PixelData"))]
    PixelDataInvalidPointer,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// PhotometricInterpretation Type for GDCM
#[derive(Debug, PartialEq, EnumString, Clone, Copy)]
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
#[derive(Debug, PartialEq, EnumString, Clone, Copy)]
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

/// Pixel data managed by GDCM
#[repr(C)]
struct PixelDataInternal {
    pixel_data: *const c_uchar,
    status: u32,
    size: size_t,
}

extern "C" {
    /// Decodes frames in GDCM
    fn c_decode_frames(
        i_buffer_ptr: *const *const c_uchar,
        i_buffer_lens: *const size_t,
        i_buffer_len: size_t,
        dims: *const u32,
        pi_type: u32,
        ts_type: u32,
        samples_per_pixel: u16,
        bits_allocated: u16,
        bits_stored: u16,
        high_bit: u16,
        pixel_representation: u16,
    ) -> PixelDataInternal;

    fn c_free_buffer(buffer_ptr: *const c_uchar);
}

/// Wraps the returned pixeldata
///
/// Exposes the data via a reference and cleans up
/// allocated data on c side on drop of the struct
pub struct PixelData {
    size: usize,
    data: NonNull<u8>,
}

impl Drop for PixelData {
    fn drop(&mut self) {
        unsafe {
            c_free_buffer(self.data.as_ptr());
        }
    }
}

impl PixelData {
    pub fn new(data: *const u8, size: usize) -> Result<Self> {
        let data = NonNull::new(data as *mut _).context(PixelDataInvalidPointerSnafu)?;
        Ok(Self { size, data })
    }
    pub fn data(&self) -> &[u8] {
        unsafe { slice::from_raw_parts_mut(self.data.as_ptr() as *mut _, self.size) }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.data().to_vec()
    }
}

/// Decodes a single frame buffer in GDCM
pub fn decode_single_frame_compressed(
    i_buffer: &[u8],
    width: u32,
    height: u32,
    pi_type: GDCMPhotometricInterpretation,
    ts_type: GDCMTransferSyntax,
    samples_per_pixel: u16,
    bits_allocated: u16,
    bits_stored: u16,
    high_bit: u16,
    pixel_representation: u16,
) -> Result<PixelData, Error> {
    let i_buffers = [i_buffer];
    let dims = [width, height, 1];
    decode_multi_frame_compressed(
        &i_buffers,
        &dims,
        pi_type,
        ts_type,
        samples_per_pixel,
        bits_allocated,
        bits_stored,
        high_bit,
        pixel_representation,
    )
}

/// Decodes a multi frame buffer in GDCM
pub fn decode_multi_frame_compressed(
    i_buffers: &[&[u8]],
    dims: &[u32; 3],
    pi_type: GDCMPhotometricInterpretation,
    ts_type: GDCMTransferSyntax,
    samples_per_pixel: u16,
    bits_allocated: u16,
    bits_stored: u16,
    high_bit: u16,
    pixel_representation: u16,
) -> Result<PixelData, Error> {
    let i_buffer_lens: Vec<usize> = i_buffers.iter().map(|fragment| fragment.len()).collect();
    let i_buffer_pointers: Vec<_> = i_buffers.iter().map(|i_buffer| i_buffer.as_ptr()).collect();
    let ret = unsafe {
        c_decode_frames(
            i_buffer_pointers.as_ptr(),
            i_buffer_lens.as_ptr(),
            i_buffers.len(),
            dims.as_ptr(),
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
        0 => PixelData::new(ret.pixel_data, ret.size),
        c => GdcmDecodingSnafu { status: c as u32 }
            .fail()
            .map_err(Error::from),
    }
}
