#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_image_constructor() {
    unsafe {
      let _image = gdcm_Image {
        _base: gdcm_Pixmap::new(),
        SC: gdcm_SwapCode {
          SwapCodeValue: gdcm_SwapCode_SwapCodeType_BigEndian
        },
        Spacing: [1, 1, 1],
        Origin: [1, 1, 1],
        DirectionCosines: [1, 1, 1],
        Intercept: 0.0,
        Slope: 1.0, 
      };
    }
  }
}