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

#[test]
fn test_image_crate_can_decode_webp() {
    use image::codecs::webp::WebPEncoder;
    use image::{ColorType, ImageEncoder};

    let width = 2u32;
    let height = 2u32;

    let rgba: Vec<u8> = vec![
        255, 0, 0, 255, // red
        0, 255, 0, 255, // green
        0, 0, 255, 255, // blue
        255, 255, 0, 255, // yellow
    ];

    let mut webp_bytes = Vec::new();
    {
        let encoder = WebPEncoder::new_lossless(&mut webp_bytes);
        encoder
            .write_image(&rgba, width, height, ColorType::Rgba8)
            .expect("encode webp");
    }

    let img = image::load_from_memory(&webp_bytes).expect("decode webp");
    assert_eq!(img.width(), width);
    assert_eq!(img.height(), height);
}

#[test]
fn test_resource_manager_creation() {
    let manager = ResourceManager::new("assets", 256);
    assert_eq!(manager.texture_count(), 0);
}

#[test]
fn test_logical_path_as_cache_key() {
    let manager = ResourceManager::new("assets", 256);

    let p = LogicalPath::new("bg.png");
    assert!(!manager.has_texture(&p));

    let p2 = LogicalPath::new("assets/bg.png");
    assert_eq!(p, p2);
}

#[test]
fn test_failed_texture_cache_can_suppress_retries() {
    let mut manager = ResourceManager::new("assets", 256);
    let p = LogicalPath::new("scripts/remake/ring/summer/bg/black");

    manager.failed_textures.insert(p.as_str().to_string());
    assert!(manager.has_failed_texture(&p));
    assert!(!manager.has_texture(&p));

    manager.unload_texture(&p);
    assert!(!manager.has_failed_texture(&p));

    manager.failed_textures.insert(p.as_str().to_string());
    manager.clear();
    assert!(!manager.has_failed_texture(&p));
}

// ---------------------------------------------------------------------------
// Headless tests (NullTextureFactory, no GPU needed)
// ---------------------------------------------------------------------------

#[test]
fn test_headless_load_texture_full_flow() {
    let mut source = InMemorySource::new();
    source.add("bg/sky.png", make_png_bytes(1920, 1080));

    let mut manager = ResourceManager::with_source(Arc::new(source), 256);
    manager.set_texture_context(TextureContext::new(Arc::new(NullTextureFactory)));

    let p = LogicalPath::new("bg/sky.png");
    let tex = manager.load_texture(&p).expect("should load");
    assert_eq!(tex.width_u32(), 1920);
    assert_eq!(tex.height_u32(), 1080);

    assert_eq!(manager.texture_count(), 1);
    assert!(manager.has_texture(&p));

    let cached = manager.peek_texture(&p);
    assert!(cached.is_some());
    assert_eq!(cached.unwrap().width_u32(), 1920);
}

#[test]
fn test_headless_load_texture_cache_hit() {
    let mut source = InMemorySource::new();
    source.add("char/a.png", make_png_bytes(512, 1024));

    let mut manager = ResourceManager::with_source(Arc::new(source), 256);
    manager.set_texture_context(TextureContext::new(Arc::new(NullTextureFactory)));

    let p = LogicalPath::new("char/a.png");
    let t1 = manager.load_texture(&p).unwrap();
    let t2 = manager.load_texture(&p).unwrap();
    assert_eq!(t1.width_u32(), t2.width_u32());

    let stats = manager.texture_cache_stats();
    assert!(stats.hits >= 1);
}

#[test]
fn test_headless_load_texture_missing_returns_error() {
    let source = InMemorySource::new();
    let mut manager = ResourceManager::with_source(Arc::new(source), 256);
    manager.set_texture_context(TextureContext::new(Arc::new(NullTextureFactory)));

    let p = LogicalPath::new("nonexistent.png");
    let result = manager.load_texture(&p);
    assert!(result.is_err());
}

#[test]
fn test_headless_no_texture_context_returns_error() {
    let mut source = InMemorySource::new();
    source.add("bg.png", make_png_bytes(64, 64));

    let mut manager = ResourceManager::with_source(Arc::new(source), 256);

    let p = LogicalPath::new("bg.png");
    let result = manager.load_texture(&p);
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Texture context not set"));
}

#[test]
fn test_read_text_optional_returns_none_for_missing() {
    let source = InMemorySource::new();
    let manager = ResourceManager::with_source(Arc::new(source), 256);

    let p = LogicalPath::new("nonexistent.json");
    assert!(manager.read_text_optional(&p).is_none());
}

#[test]
fn test_read_text_optional_returns_content() {
    let mut source = InMemorySource::new();
    source.add("config.json", b"{\"key\": \"value\"}".to_vec());

    let manager = ResourceManager::with_source(Arc::new(source), 256);

    let p = LogicalPath::new("config.json");
    let result = manager.read_text_optional(&p);
    assert!(result.is_some());
    assert!(result.unwrap().contains("key"));
}
