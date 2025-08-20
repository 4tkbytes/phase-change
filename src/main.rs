use std::path::PathBuf;

use phase_change::{converters::image::ImageFileType, FileType};

fn main() -> anyhow::Result<()> {
    let mut builder = phase_change::FileConvertBuilder::new();
    builder.from_file(FileType::Image(ImageFileType::PNG), PathBuf::from("resources/input.png"));
    builder.to_file(FileType::Image(ImageFileType::JPEG), Some(PathBuf::from("output.jpg")));
    builder.convert()?;

    Ok(())
}
