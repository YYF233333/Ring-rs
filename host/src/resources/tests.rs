use super::*;

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
    assert_eq!(manager.sound_count(), 0);
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
