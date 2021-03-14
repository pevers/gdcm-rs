use libc::{c_uchar, c_uint, c_void, size_t};
use std::slice;

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
    width: u32,
    height: u32,
    pi_type: u32,
    ts_type: u32,
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
            pi_type,
            ts_type,
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
    use dicom_core::value::Value;
    use dicom_object::open_file;
    use dicom_test_files;
    use image::{ImageBuffer, Luma};
    use std::{fs::File, io::Read, slice};

    use crate::{decode_single_frame_compressed, decode_dicom_file};
    #[test]
    fn test_read_jpeg_lossy() {
        let width = 1024;
        let height = 768;
        let mut file =
            File::open(dicom_test_files::path("pydicom/JPGLosslessP14SV1_1s_1f_8b.dcm").unwrap())
                .unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        let pixel_data: Vec<u8> = decode_dicom_file(buffer).unwrap().into();
        let slice: ImageBuffer<Luma<u8>, Vec<u8>> =
            ImageBuffer::from_raw(width, height, pixel_data).unwrap();
        slice.save("target/test_read_jpeg_lossy.png").unwrap();
    }

    #[test]
    fn test_decode_in_memory_pixel_data_single_frame() {
        let obj =
            open_file(dicom_test_files::path("pydicom/JPGLosslessP14SV1_1s_1f_8b.dcm").unwrap())
                .unwrap();
        let pixel_data = obj.element_by_name("PixelData").unwrap();
        let width: u32 = obj.element_by_name("Columns").unwrap().to_int().unwrap();
        let height: u32 = obj.element_by_name("Rows").unwrap().to_int().unwrap();
        let pi_type = 2; // MONOCHROME2
        let ts_type = 11; // JPEGLosslessProcess14_1
        let samples_per_pixel: u16 = 1;
        let bits_allocated: u16 = 8;
        let bits_stored: u16 = 8;
        let high_bit: u16 = 7;
        let pixel_representation: u16 = 0;

        match pixel_data.value() {
            Value::PixelSequence {
                fragments,
                offset_table,
            } => {
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
                let decoded_pixel_data: Vec<u8> = ret.unwrap().into();
                let slice: ImageBuffer<Luma<u8>, Vec<u8>> =
                    ImageBuffer::from_raw(width, height, decoded_pixel_data).unwrap();
                slice.save("target/in_memory_slice.png").unwrap();
            },
            _ => panic!("Error, no pixel sequence"),
        }
    }
}
