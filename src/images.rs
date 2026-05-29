use crate::core::SizeU32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecodedImage {
    pub size: SizeU32,
    pub rgba: Vec<u8>,
}

pub fn decode_rgba(bytes: &[u8]) -> Result<DecodedImage, image::ImageError> {
    let image = image::load_from_memory(bytes)?.to_rgba8();
    let (width, height) = image.dimensions();
    Ok(DecodedImage {
        size: SizeU32::new(width, height),
        rgba: image.into_raw(),
    })
}
