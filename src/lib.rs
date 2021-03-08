use libc::{c_uchar, c_uint, size_t};
use std::slice;

#[repr(C)]
struct pixel_data_output_t {
    pixel_data: *const c_uchar,
    status: c_uint,
    size: size_t,
}

extern "C" {
    fn c_get_pixel_data(i_buffer_ptr: *const c_uchar, i_buffer_len: size_t) -> pixel_data_output_t;
}

/// Takes as input a DICOM and return the raw decoded pixel data
/// Temporary solution, we should accept the pixel data extracted by dicom-rs
pub fn get_pixel_data(i_buffer: Vec<u8>) -> Result<Box<[u8]>, &'static str> {
    let ret = unsafe { c_get_pixel_data(i_buffer.as_ptr(), i_buffer.len() as size_t) };
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

#[cfg(test)]
mod tests {
    use dicom_test_files;
    use image::{ImageBuffer, Luma};
    use std::{fs::File, io::Read};

    use crate::get_pixel_data;

    // Reads a lossless JPEG and converts it to a PNG
    #[test]
    fn test_read_jpeg_lossy() {
        let width = 1024;
        let height = 768;
        let mut file =
            File::open(dicom_test_files::path("pydicom/JPGLosslessP14SV1_1s_1f_8b.dcm").unwrap())
                .unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        let pixel_data: Vec<u8> = get_pixel_data(buffer).unwrap().into();
        let slice: ImageBuffer<Luma<u8>, Vec<u8>> =
            ImageBuffer::from_raw(width, height, pixel_data).unwrap();
        slice.save("target/test_read_jpeg_lossy.png").unwrap();
    }
}
