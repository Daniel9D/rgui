use crate::core::SizeU32;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RasterizedSvg {
    pub size: SizeU32,
    pub rgba: Vec<u8>,
}

pub fn rasterize_svg_bytes(bytes: &[u8], size: SizeU32) -> Result<RasterizedSvg, String> {
    if bytes.is_empty() {
        return Err("svg input is empty".to_string());
    }
    Ok(RasterizedSvg {
        size,
        rgba: vec![0; size.width as usize * size.height as usize * 4],
    })
}
