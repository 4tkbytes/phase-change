use std::path::PathBuf;

use crate::{converters::image::ImageFileType, Converter, FileType};

pub struct PngToJpeg;

impl Converter for PngToJpeg {
    fn convert(&self, input_path: &PathBuf, output_path: &PathBuf) -> anyhow::Result<()> {        
        let img = image::open(input_path)?;
        let img = img.to_rgb8();
        let mut output = std::fs::File::create(output_path)?;
        let encoder = image::codecs::jpeg::JpegEncoder::new(&mut output);
        img.write_with_encoder(encoder)?;
        
        std::fs::copy(input_path, output_path)?;
        Ok(())
    }

    fn from_type(&self) -> FileType {
        FileType::Image(ImageFileType::PNG)
    }

    fn to_type(&self) -> FileType {
        FileType::Image(ImageFileType::JPEG)
    }
}