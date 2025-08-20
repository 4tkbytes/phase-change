pub mod png;

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
pub enum ImageFileType {
    PNG,
    JPEG,
}