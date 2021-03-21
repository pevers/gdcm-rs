use byteorder::{BigEndian, ByteOrder};
use image::{DynamicImage, ImageBuffer, Luma};
use snafu::Snafu;

/// This file will be moved to its own crate dicom-pixel-data

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Bits allocated {} is not supported", bits_allocated))]
    UnsupportedPixelRepresentation { bits_allocated: u16 },
}

/// Converts a &Vec<u8> into a DynamicImage based on the bits_allocated
/// Note: No RGB support (yet)
pub fn encode_pixeldata(
    buffer: &Vec<u8>,
    width: u16,
    height: u16,
    bits_allocated: u16,
) -> Result<DynamicImage, Error> {
    // TODO: Take 2s complement into account! Look at the j2k examples, they are wrong!

    match bits_allocated {
        8 => {
            let image_buffer: ImageBuffer<Luma<u8>, Vec<u8>> =
                ImageBuffer::from_raw(width.into(), height.into(), buffer.to_owned()).unwrap();
            Ok(DynamicImage::from(DynamicImage::ImageLuma8(image_buffer)))
        }
        16 => {
            let mut dest = vec![0; buffer.len() / 2];
            BigEndian::read_u16_into(&buffer, &mut dest);
            let image_buffer: ImageBuffer<Luma<u16>, Vec<u16>> =
                ImageBuffer::from_raw(width.into(), height.into(), dest).unwrap();
            Ok(DynamicImage::from(DynamicImage::ImageLuma16(image_buffer)))
        }
        _ => Err(Error::UnsupportedPixelRepresentation { bits_allocated }),
    }
}

// TODO: Tests
