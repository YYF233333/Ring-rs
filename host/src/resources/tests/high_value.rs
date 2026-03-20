use super::*;

#[test]
fn test_image_crate_can_decode_webp() {
    use image::codecs::webp::WebPEncoder;
    use image::{ColorType, ImageEncoder};

    let width = 2u32;
    let height = 2u32;

    let rgba: Vec<u8> = vec![
        255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 0, 255,
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
    assert_eq!(manager.texture_count(), 1);
    assert!(manager.has_texture(&p));
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

#[test]
fn test_read_text_success() {
    let mut source = InMemorySource::new();
    source.add("data.txt", b"hello world".to_vec());

    let manager = ResourceManager::with_source(Arc::new(source), 256);
    let p = LogicalPath::new("data.txt");
    assert_eq!(manager.read_text(&p).unwrap(), "hello world");
}

#[test]
fn test_read_text_missing_returns_error() {
    let source = InMemorySource::new();
    let manager = ResourceManager::with_source(Arc::new(source), 256);
    let p = LogicalPath::new("missing.txt");
    assert!(manager.read_text(&p).is_err());
}

#[test]
fn test_read_bytes_success() {
    let mut source = InMemorySource::new();
    source.add("raw.bin", vec![0xDE, 0xAD, 0xBE, 0xEF]);

    let manager = ResourceManager::with_source(Arc::new(source), 256);
    let p = LogicalPath::new("raw.bin");
    assert_eq!(
        manager.read_bytes(&p).unwrap(),
        vec![0xDE, 0xAD, 0xBE, 0xEF]
    );
}

#[test]
fn test_resource_exists() {
    let mut source = InMemorySource::new();
    source.add("exists.png", b"data".to_vec());

    let manager = ResourceManager::with_source(Arc::new(source), 256);
    assert!(manager.resource_exists(&LogicalPath::new("exists.png")));
    assert!(!manager.resource_exists(&LogicalPath::new("ghost.png")));
}

#[test]
fn test_list_files_delegates_to_source() {
    let mut source = InMemorySource::new();
    source.add("bg/sky.png", b"data".to_vec());
    source.add("bg/ocean.png", b"data".to_vec());

    let manager = ResourceManager::with_source(Arc::new(source), 256);
    let files = manager.list_files(&LogicalPath::new("bg"));
    assert_eq!(files.len(), 2);
}

#[test]
fn test_unload_texture_clears_failed_flag() {
    let source = InMemorySource::new();
    let mut manager = ResourceManager::with_source(Arc::new(source), 256);
    let p = LogicalPath::new("bad.png");

    let _ = manager.load_texture(&p);
    assert!(manager.has_failed_texture(&p));

    manager.unload_texture(&p);
    assert!(!manager.has_failed_texture(&p));
}

#[test]
fn test_preload_textures_success() {
    let mut source = InMemorySource::new();
    source.add("bg/a.png", make_png_bytes(64, 64));
    source.add("bg/b.png", make_png_bytes(32, 32));

    let mut manager = ResourceManager::with_source(Arc::new(source), 256);
    manager.set_texture_context(TextureContext::new(Arc::new(NullTextureFactory)));

    let a = LogicalPath::new("bg/a.png");
    let b = LogicalPath::new("bg/b.png");
    manager.preload_textures(&[&a, &b]).unwrap();
    assert_eq!(manager.texture_count(), 2);
}

#[test]
fn test_preload_textures_stops_on_first_error() {
    let mut source = InMemorySource::new();
    source.add("ok.png", make_png_bytes(32, 32));

    let mut manager = ResourceManager::with_source(Arc::new(source), 256);
    manager.set_texture_context(TextureContext::new(Arc::new(NullTextureFactory)));

    let ok = LogicalPath::new("ok.png");
    let bad = LogicalPath::new("bad.png");
    let result = manager.preload_textures(&[&bad, &ok]);
    assert!(result.is_err());
}

#[test]
fn test_load_failed_texture_suppresses_retry() {
    let source = InMemorySource::new();
    let mut manager = ResourceManager::with_source(Arc::new(source), 256);
    manager.set_texture_context(TextureContext::new(Arc::new(NullTextureFactory)));

    let p = LogicalPath::new("missing.png");

    assert!(manager.load_texture(&p).is_err());
    assert!(manager.has_failed_texture(&p));

    let err = manager.load_texture(&p).unwrap_err();
    assert!(format!("{err}").contains("Previously failed") || manager.has_failed_texture(&p));
}

#[test]
fn test_texture_cache_stats_after_load() {
    let mut source = InMemorySource::new();
    source.add("bg.png", make_png_bytes(256, 256));

    let mut manager = ResourceManager::with_source(Arc::new(source), 256);
    manager.set_texture_context(TextureContext::new(Arc::new(NullTextureFactory)));

    let p = LogicalPath::new("bg.png");
    manager.load_texture(&p).unwrap();

    let stats = manager.texture_cache_stats();
    assert_eq!(stats.entries, 1);
    assert!(stats.used_bytes > 0);
}

#[test]
fn test_materialize_to_fs_zip_source() {
    use crate::resources::source::ZipSource;
    use std::io::Write as _;

    let tmp = tempfile::tempdir().unwrap();
    let zip_path = tmp.path().join("pack.zip");
    {
        let file = std::fs::File::create(&zip_path).unwrap();
        let mut zip = zip::ZipWriter::new(file);
        let opts = zip::write::SimpleFileOptions::default();
        zip.start_file("sound/bgm.ogg", opts).unwrap();
        zip.write_all(b"fake-ogg").unwrap();
        zip.finish().unwrap();
    }

    let source = Arc::new(ZipSource::new(&zip_path));
    let manager = ResourceManager::with_source(source, 256);

    let p = LogicalPath::new("sound/bgm.ogg");
    let temp_out = tmp.path().join("temp_out");
    let (out_path, cleanup) = manager.materialize_to_fs(&p, &temp_out).unwrap();

    assert!(out_path.exists());
    assert_eq!(std::fs::read(&out_path).unwrap(), b"fake-ogg");
    assert!(cleanup.is_some());
}

#[test]
fn test_logical_path_as_cache_key() {
    let manager = ResourceManager::new("assets", 256);

    let p = LogicalPath::new("bg.png");
    assert!(!manager.has_texture(&p));

    let p2 = LogicalPath::new("assets/bg.png");
    assert_eq!(p, p2);
}
