use super::*;
use crate::rendering_types::{NullTextureFactory, TextureContext};

fn make_png_bytes(width: u32, height: u32) -> Vec<u8> {
    use image::{ImageBuffer, Rgba};
    let img = ImageBuffer::from_pixel(width, height, Rgba([255u8, 0, 0, 255]));
    let mut buf = std::io::Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::Png).unwrap();
    buf.into_inner()
}

struct InMemorySource {
    files: std::collections::HashMap<String, Vec<u8>>,
}

impl InMemorySource {
    fn new() -> Self {
        Self {
            files: std::collections::HashMap::new(),
        }
    }

    fn add(&mut self, path: &str, data: Vec<u8>) {
        self.files.insert(path.to_string(), data);
    }
}

impl ResourceSource for InMemorySource {
    fn read(&self, path: &LogicalPath) -> Result<Vec<u8>, ResourceError> {
        self.files
            .get(path.as_str())
            .cloned()
            .ok_or(ResourceError::NotFound {
                path: path.to_string(),
            })
    }

    fn exists(&self, path: &LogicalPath) -> bool {
        self.files.contains_key(path.as_str())
    }

    fn full_path(&self, path: &LogicalPath) -> String {
        format!("memory://{}", path)
    }

    fn list_files(&self, dir_path: &LogicalPath) -> Vec<LogicalPath> {
        self.files
            .keys()
            .filter(|k| k.starts_with(dir_path.as_str()))
            .map(|k| LogicalPath::new(k))
            .collect()
    }
}
mod high_value;
