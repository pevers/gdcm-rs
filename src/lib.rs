mod encode_image;
use libc::{c_uchar, c_uint, c_void, size_t};
use snafu::{ensure, Backtrace, ErrorCompat, ResultExt, Snafu};
use std::slice;
use strum_macros::EnumString;

/// PhotometricInterpretation Type
#[derive(Debug, PartialEq, EnumString)]
#[allow(non_camel_case_types)]
pub enum PhotometricInterpretation {
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

/// GDCM TransferSyntax encodings
#[derive(Debug, PartialEq, EnumString)]
#[allow(non_camel_case_types)]
pub enum TransferSyntax {
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

/// TransferSyntax Type
pub enum TSType {}

#[repr(C)]
struct pixel_data {
    pixel_data: *const c_uchar,
    status: c_uint,
    size: size_t,
}

#[repr(C)]
struct sequence_of_fragments {
    ptr: *mut c_void,
}

extern "C" {
    fn c_decode_dicom_file(i_buffer_ptr: *const c_uchar, i_buffer_len: size_t) -> pixel_data;
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
        width: u16,
        height: u16,
        pi_type: u32,
        ts_type: u32,
        samples_per_pixel: u16,
        bits_allocated: u16,
        bits_stored: u16,
        high_bit: u16,
        pixel_representation: u16,
    ) -> pixel_data;
}

/// Takes as input a DICOM and return the raw decoded pixel data
pub fn decode_dicom_file(i_buffer: Vec<u8>) -> Result<Box<[u8]>, &'static str> {
    let ret = unsafe { c_decode_dicom_file(i_buffer.as_ptr(), i_buffer.len() as size_t) };
    match ret.status {
        0 => unsafe {
            let slice = slice::from_raw_parts_mut(ret.pixel_data as *mut _, ret.size);
            return Ok(Box::from_raw(slice));
        },
        1 => return Err("Can't allocate size."),
        2 => return Err("Can't read stream."),
        _ => return Err("Unknown error."),
    }
}

/// Takes an in-memory dicom-rs PixelData and decodes it
pub fn decode_single_frame_compressed(
    i_buffer: &Vec<u8>,
    width: u16,
    height: u16,
    pi_type: PhotometricInterpretation,
    ts_type: TransferSyntax,
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

#[cfg(test)]
mod tests {
    use super::*;
    use dicom_core::value::{DicomValueType, Value, ValueType};
    use dicom_encoding::transfer_syntax::TransferSyntaxIndex;
    use dicom_object::open_file;
    use dicom_test_files;
    use dicom_transfer_syntax_registry::TransferSyntaxRegistry;
    use encode_image::encode_pixeldata;
    use rstest::rstest;
    use std::{path::Path, str::FromStr};

    use crate::decode_single_frame_compressed;

    #[rstest(value => [
        // "pydicom/693_J2KI.dcm",
        // "pydicom/693_J2KR.dcm",
        // "pydicom/693_UNCI.dcm",
        // "pydicom/693_UNCR.dcm",
        // "pydicom/CT_small.dcm",
        // "pydicom/ExplVR_BigEnd.dcm",
        // "pydicom/ExplVR_BigEndNoMeta.dcm",
        // "pydicom/ExplVR_LitEndNoMeta.dcm",
        // "pydicom/JPEG-LL.dcm",               // More than 1 fragment
        // "pydicom/JPEG-lossy.dcm",
        // "pydicom/JPEG2000.dcm",
        "pydicom/JPEG2000_UNC.dcm",
        // "pydicom/JPGLosslessP14SV1_1s_1f_8b.dcm",

        // "pydicom/MR-SIEMENS-DICOM-WithOverlays.dcm",
        // "pydicom/MR2_J2KI.dcm",
        // "pydicom/MR2_J2KR.dcm",
        // "pydicom/MR2_UNCI.dcm",
        // "pydicom/MR2_UNCR.dcm",
        // "pydicom/MR_small.dcm",
        // "pydicom/MR_small_RLE.dcm",
        // "pydicom/MR_small_bigendian.dcm",
        // "pydicom/MR_small_expb.dcm",
        // "pydicom/MR_small_implicit.dcm",
        // "pydicom/MR_small_jp2klossless.dcm",
        // "pydicom/MR_small_jpeg_ls_lossless.dcm",
        // "pydicom/MR_small_padded.dcm",
        // "pydicom/MR_truncated.dcm",
        // "pydicom/OBXXXX1A.dcm",
        // "pydicom/OBXXXX1A_2frame.dcm",
        // "pydicom/OBXXXX1A_expb.dcm",
        // "pydicom/OBXXXX1A_expb_2frame.dcm",
        // "pydicom/OBXXXX1A_rle.dcm",
        // "pydicom/OBXXXX1A_rle_2frame.dcm",
        // "pydicom/OT-PAL-8-face.dcm",
        // "pydicom/README.txt",
        // "pydicom/RG1_J2KI.dcm",
        // "pydicom/RG1_J2KR.dcm",
        // "pydicom/RG1_UNCI.dcm",
        // "pydicom/RG1_UNCR.dcm",
        // "pydicom/RG3_J2KI.dcm",
        // "pydicom/RG3_J2KR.dcm",
        // "pydicom/RG3_UNCI.dcm",
        // "pydicom/RG3_UNCR.dcm",
        // "pydicom/SC_rgb.dcm",
        // "pydicom/SC_rgb_16bit.dcm",
        // "pydicom/SC_rgb_16bit_2frame.dcm",
        // "pydicom/SC_rgb_2frame.dcm",
        // "pydicom/SC_rgb_32bit.dcm",
        // "pydicom/SC_rgb_32bit_2frame.dcm",
        // "pydicom/SC_rgb_dcmtk_+eb+cr.dcm",
        // "pydicom/SC_rgb_dcmtk_+eb+cy+n1.dcm",
        // "pydicom/SC_rgb_dcmtk_+eb+cy+n2.dcm",
        // "pydicom/SC_rgb_dcmtk_+eb+cy+np.dcm",
        // "pydicom/SC_rgb_dcmtk_+eb+cy+s2.dcm",
        // "pydicom/SC_rgb_dcmtk_+eb+cy+s4.dcm",
        // "pydicom/SC_rgb_dcmtk_ebcr_dcmd.dcm",
        // "pydicom/SC_rgb_dcmtk_ebcyn1_dcmd.dcm",
        // "pydicom/SC_rgb_dcmtk_ebcyn2_dcmd.dcm",
        // "pydicom/SC_rgb_dcmtk_ebcynp_dcmd.dcm",
        // "pydicom/SC_rgb_dcmtk_ebcys2_dcmd.dcm",
        // "pydicom/SC_rgb_dcmtk_ebcys4_dcmd.dcm",
        // "pydicom/SC_rgb_expb.dcm",
        // "pydicom/SC_rgb_expb_16bit.dcm",
        // "pydicom/SC_rgb_expb_16bit_2frame.dcm",
        // "pydicom/SC_rgb_expb_2frame.dcm",
        // "pydicom/SC_rgb_expb_32bit.dcm",
        // "pydicom/SC_rgb_expb_32bit_2frame.dcm",
        // "pydicom/SC_rgb_gdcm2k_uncompressed.dcm",
        // "pydicom/SC_rgb_gdcm_KY.dcm",
        // "pydicom/SC_rgb_jpeg_dcmtk.dcm",
        // "pydicom/SC_rgb_jpeg_gdcm.dcm",
        // "pydicom/SC_rgb_jpeg_lossy_gdcm.dcm",
        // "pydicom/SC_rgb_rle.dcm",
        // "pydicom/SC_rgb_rle_16bit.dcm",
        // "pydicom/SC_rgb_rle_16bit_2frame.dcm",
        // "pydicom/SC_rgb_rle_2frame.dcm",
        // "pydicom/SC_rgb_rle_32bit.dcm",
        // "pydicom/SC_rgb_rle_32bit_2frame.dcm",
        // "pydicom/SC_rgb_small_odd.dcm",
        // "pydicom/SC_rgb_small_odd_jpeg.dcm",
        // "pydicom/SC_ybr_full_422_uncompressed.dcm",
        // "pydicom/SC_ybr_full_uncompressed.dcm",
        // "pydicom/US1_J2KI.dcm",
        // "pydicom/US1_J2KR.dcm",
        // "pydicom/US1_UNCI.dcm",
        // "pydicom/US1_UNCR.dcm",
        // "pydicom/badVR.dcm",
        // "pydicom/bad_sequence.dcm",
        // "pydicom/color-pl.dcm",
        // "pydicom/color-px.dcm",
        // "pydicom/color3d_jpeg_baseline.dcm",
        // "pydicom/eCT_Supplemental.dcm",
        // "pydicom/empty_charset_LEI.dcm",
        // "pydicom/emri_small.dcm",
        // "pydicom/emri_small_RLE.dcm",
        // "pydicom/emri_small_big_endian.dcm",
        // "pydicom/emri_small_jpeg_2k_lossless.dcm",
        // "pydicom/emri_small_jpeg_2k_lossless_too_short.dcm",
        // "pydicom/emri_small_jpeg_ls_lossless.dcm",
        // "pydicom/explicit_VR-UN.dcm",
        // "pydicom/gdcm-US-ALOKA-16.dcm",
        // "pydicom/gdcm-US-ALOKA-16_big.dcm",
        // "pydicom/image_dfl.dcm",
        // "pydicom/liver.dcm",
        // "pydicom/liver_1frame.dcm",
        // "pydicom/liver_expb.dcm",
        // "pydicom/liver_expb_1frame.dcm",
        // "pydicom/meta_missing_tsyntax.dcm",
        // "pydicom/mlut_18.dcm",
        // "pydicom/nested_priv_SQ.dcm",
        // "pydicom/no_meta.dcm",
        // "pydicom/no_meta_group_length.dcm",
        // "pydicom/priv_SQ.dcm",
        // "pydicom/reportsi.dcm",
        // "pydicom/reportsi_with_empty_number_tags.dcm",
        // "pydicom/rtdose.dcm",
        // "pydicom/rtdose_1frame.dcm",
        // "pydicom/rtdose_expb.dcm",
        // "pydicom/rtdose_expb_1frame.dcm",
        // "pydicom/rtdose_rle.dcm",
        // "pydicom/rtdose_rle_1frame.dcm",
        // "pydicom/rtplan.dcm",
        // "pydicom/rtplan_truncated.dcm",
        // "pydicom/rtstruct.dcm",
        // "pydicom/test-SR.dcm",
        // "pydicom/vlut_04.dcm",
    ])]
    fn test_parse_dicom_pixel_data(value: &str) {
        println!("Parsing pixel data for {}", value);
        let obj = open_file(dicom_test_files::path(value).unwrap()).unwrap();
        let pixel_data = obj.element_by_name("PixelData").unwrap();
        let width: u16 = obj.element_by_name("Columns").unwrap().uint16().unwrap();
        let height: u16 = obj.element_by_name("Rows").unwrap().uint16().unwrap();
        let photometric = obj
            .element_by_name("PhotometricInterpretation")
            .unwrap()
            .to_str()
            .unwrap();
        let pi_type = PhotometricInterpretation::from_str(&photometric.trim()).expect(&format!(
            "PhotometricInterpretation {} not found",
            photometric
        ));
        let transfer_syntax = &obj.meta().transfer_syntax;
        let ts_type =
            TransferSyntax::from_str(TransferSyntaxRegistry.get(&transfer_syntax).unwrap().uid())
                .expect(&format!(
                    "Transfer syntax {} not supported",
                    transfer_syntax
                ));
        let samples_per_pixel: u16 = obj
            .element_by_name("SamplesPerPixel")
            .unwrap()
            .uint16()
            .unwrap();
        if samples_per_pixel > 1 {
            panic!("No support yet for more than 1 SamplesPerPixel");
        }
        let bits_allocated: u16 = obj
            .element_by_name("BitsAllocated")
            .unwrap()
            .uint16()
            .unwrap();
        let bits_stored: u16 = obj.element_by_name("BitsStored").unwrap().uint16().unwrap();
        let high_bit: u16 = obj.element_by_name("HighBit").unwrap().uint16().unwrap();
        let pixel_representation: u16 = obj
            .element_by_name("PixelRepresentation")
            .unwrap()
            .uint16()
            .unwrap();

        match pixel_data.value() {
            Value::PixelSequence {
                fragments,
                offset_table: _,
            } => {
                if fragments.len() > 1 {
                    panic!("More than 1 fragment found, not yet supported!");
                }
                let ret = decode_single_frame_compressed(
                    &fragments[0],
                    width,
                    height,
                    pi_type,
                    ts_type,
                    samples_per_pixel,
                    bits_allocated,
                    bits_stored,
                    high_bit,
                    pixel_representation,
                );
                let image = encode_pixeldata(&ret.unwrap().to_vec(), width, height, bits_allocated)
                    .unwrap();
                image
                    .save(format!(
                        "target/{}.png",
                        Path::new(value).file_stem().unwrap().to_str().unwrap()
                    ))
                    .unwrap();
            }
            Value::Primitive(p) => {
                // TODO: Figure out why this is a PrimitiveValue instead of PixelSequence
                match p.value_type() {
                    ValueType::U16 => {
                        let image =
                            encode_pixeldata(&p.to_bytes().to_vec(), width, height, 16).unwrap();
                        image
                            .save(format!(
                                "target/{}.png",
                                Path::new(value).file_stem().unwrap().to_str().unwrap()
                            ))
                            .unwrap();
                    }
                    _ => panic!("Unsupported value type {:?}", p),
                }
                panic!(
                    "Found a primitive value with value type {:?}",
                    p.value_type()
                );
            }
            Value::Sequence { items, size } => {
                panic!("Found a sequence instead of a PixelSequence {:?}", size);
            }
        }
    }
}
