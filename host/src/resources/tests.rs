use super::*;
use crate::rendering_types::{NullTextureFactory, TextureContext};

/// 生成最小合法 PNG 字节（用于 headless 测试）
fn make_png_bytes(width: u32, height: u32) -> Vec<u8> {
    use image::{ImageBuffer, Rgba};
    let img = ImageBuffer::from_pixel(width, height, Rgba([255u8, 0, 0, 255]));
    let mut buf = std::io::Cursor::new(Vec::new());
    img.write_to(&mut buf, image::ImageFormat::Png).unwrap();
    buf.into_inner()
}

/// 内存资源源（headless 测试用）
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
    fn read(&self, path: &str) -> Result<Vec<u8>, ResourceError> {
        self.files
            .get(path)
            .cloned()
            .ok_or(ResourceError::NotFound {
                path: path.to_string(),
            })
    }

    fn exists(&self, path: &str) -> bool {
        self.files.contains_key(path)
    }

    fn full_path(&self, path: &str) -> String {
        format!("memory://{}", path)
    }

    fn list_files(&self, dir_path: &str) -> Vec<String> {
        self.files
            .keys()
            .filter(|k| k.starts_with(dir_path))
            .cloned()
            .collect()
    }
}

#[test]
fn test_image_crate_can_decode_webp() {
    use image::codecs::webp::WebPEncoder;
    use image::{ColorType, ImageEncoder};

    let width = 2u32;
    let height = 2u32;

    // RGBA8: 2x2
    let rgba: Vec<u8> = vec![
        255, 0, 0, 255, // red
        0, 255, 0, 255, // green
        0, 0, 255, 255, // blue
        255, 255, 0, 255, // yellow
    ];

    let mut webp_bytes = Vec::new();
    {
        // lossless 编码，避免有损压缩带来的像素差异
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
fn test_resolve_path() {
    let manager = ResourceManager::new("assets", 256);

    // 相对路径
    let path = manager.resolve_path("bg.png");
    assert_eq!(path, "assets/bg.png");

    // 绝对路径（包含 assets）
    let path = manager.resolve_path("assets/bg.png");
    assert_eq!(path, "assets/bg.png");
}

#[test]
fn test_failed_texture_cache_can_suppress_retries() {
    let mut manager = ResourceManager::new("assets", 256);
    let missing = "scripts/remake/ring/summer/bg/black";
    let full = manager.resolve_path(missing);

    manager.failed_textures.insert(full.clone());
    assert!(manager.has_failed_texture(missing));
    assert!(!manager.has_texture(missing));

    manager.unload_texture(missing);
    assert!(!manager.has_failed_texture(missing));

    manager.failed_textures.insert(full);
    manager.clear();
    assert!(!manager.has_failed_texture(missing));
}

// ---------------------------------------------------------------------------
// Headless 测试（使用 NullTextureFactory，无需 GPU）
// ---------------------------------------------------------------------------

#[test]
fn test_headless_load_texture_full_flow() {
    let mut source = InMemorySource::new();
    source.add("bg/sky.png", make_png_bytes(1920, 1080));

    let mut manager = ResourceManager::with_source("", Arc::new(source), 256);
    manager.set_texture_context(TextureContext::new(Arc::new(NullTextureFactory)));

    let tex = manager.load_texture("bg/sky.png").expect("should load");
    assert_eq!(tex.width_u32(), 1920);
    assert_eq!(tex.height_u32(), 1080);

    assert_eq!(manager.texture_count(), 1);
    assert!(manager.has_texture("bg/sky.png"));

    let cached = manager.peek_texture("bg/sky.png");
    assert!(cached.is_some());
    assert_eq!(cached.unwrap().width_u32(), 1920);
}

#[test]
fn test_headless_load_texture_cache_hit() {
    let mut source = InMemorySource::new();
    source.add("char/a.png", make_png_bytes(512, 1024));

    let mut manager = ResourceManager::with_source("", Arc::new(source), 256);
    manager.set_texture_context(TextureContext::new(Arc::new(NullTextureFactory)));

    let t1 = manager.load_texture("char/a.png").unwrap();
    let t2 = manager.load_texture("char/a.png").unwrap();
    assert_eq!(t1.width_u32(), t2.width_u32());

    let stats = manager.texture_cache_stats();
    assert!(stats.hits >= 1);
}

#[test]
fn test_headless_load_texture_missing_returns_error() {
    let source = InMemorySource::new();
    let mut manager = ResourceManager::with_source("", Arc::new(source), 256);
    manager.set_texture_context(TextureContext::new(Arc::new(NullTextureFactory)));

    let result = manager.load_texture("nonexistent.png");
    assert!(result.is_err());
}

#[test]
fn test_headless_no_texture_context_returns_error() {
    let mut source = InMemorySource::new();
    source.add("bg.png", make_png_bytes(64, 64));

    let mut manager = ResourceManager::with_source("", Arc::new(source), 256);
    // 不注入 TextureContext

    let result = manager.load_texture("bg.png");
    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Texture context not set"));
}
