use base64::{engine::general_purpose::STANDARD, Engine};
use image::{DynamicImage, GenericImageView, ImageFormat, ImageReader};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    io::Cursor,
    path::{Path, PathBuf},
};
use tauri::{AppHandle, Manager};

const BUILTIN_MANIFEST: &str = include_str!("../resources/pets/pathlight/pet.json");
const BUILTIN_SPRITESHEET: &[u8] = include_bytes!("../resources/pets/pathlight/spritesheet.webp");
const SPIDER_MAN_SELECTION: &str = "builtin:spiderman4";
const SPIDER_MAN_MANIFEST: &str = include_str!("../resources/pets/spiderman4/pet.json");
const SPIDER_MAN_SPRITESHEET: &[u8] =
    include_bytes!("../resources/pets/spiderman4/spritesheet.png");

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CodexPetManifest {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub spritesheet_path: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PetChoice {
    pub manifest_path: String,
    pub id: String,
    pub display_name: String,
    pub description: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PetPayload {
    pub manifest: CodexPetManifest,
    pub spritesheet_data_url: String,
    pub frame_width: u32,
    pub frame_height: u32,
    pub columns: u32,
    pub rows: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PetThumbnail {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub image_data_url: String,
}

pub fn builtin_payload() -> Result<PetPayload, String> {
    let manifest = parse_manifest(BUILTIN_MANIFEST)?;
    payload(manifest, BUILTIN_SPRITESHEET, "image/webp")
}

fn spider_man_payload() -> Result<PetPayload, String> {
    let manifest = parse_manifest(SPIDER_MAN_MANIFEST)?;
    payload(manifest, SPIDER_MAN_SPRITESHEET, "image/png")
}

pub fn selected_payload(selection: Option<&str>) -> Result<PetPayload, String> {
    match selection {
        Some(SPIDER_MAN_SELECTION) => spider_man_payload(),
        Some(path) => load_payload(Path::new(path)),
        None => builtin_payload(),
    }
}

pub fn selected_thumbnail(selection: Option<&str>) -> Result<PetThumbnail, String> {
    let (manifest, image) = match selection {
        Some(SPIDER_MAN_SELECTION) => (
            parse_manifest(SPIDER_MAN_MANIFEST)?,
            SPIDER_MAN_SPRITESHEET.to_vec(),
        ),
        Some(path) => {
            let path = Path::new(path);
            let raw = fs::read_to_string(path).map_err(|error| error.to_string())?;
            let manifest = parse_manifest(&raw)?;
            let directory = path
                .parent()
                .ok_or_else(|| "pet.json 必须位于宠物资源目录中".to_string())?;
            let sprite_path = directory.join(&manifest.spritesheet_path);
            if !sprite_path.starts_with(directory) {
                return Err("spritesheetPath 不能离开宠物资源目录".into());
            }
            let image = fs::read(sprite_path).map_err(|error| error.to_string())?;
            (manifest, image)
        }
        None => (
            parse_manifest(BUILTIN_MANIFEST)?,
            BUILTIN_SPRITESHEET.to_vec(),
        ),
    };
    let image_data_url = representative_frame_data_url(&image)?;
    Ok(PetThumbnail {
        id: manifest.id,
        display_name: manifest.display_name,
        description: manifest.description,
        image_data_url,
    })
}

pub fn load_payload(path: &Path) -> Result<PetPayload, String> {
    let raw = fs::read_to_string(path).map_err(|error| error.to_string())?;
    let manifest = parse_manifest(&raw)?;
    let directory = path
        .parent()
        .ok_or_else(|| "pet.json 必须位于宠物资源目录中".to_string())?;
    let sprite_path = directory.join(&manifest.spritesheet_path);
    if !sprite_path.starts_with(directory) {
        return Err("spritesheetPath 不能离开宠物资源目录".into());
    }
    let image = fs::read(&sprite_path).map_err(|error| error.to_string())?;
    let mime = match sprite_path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => "image/png",
        "webp" => "image/webp",
        _ => return Err("序列帧图集必须是 .webp 或 .png".into()),
    };
    payload(manifest, &image, mime)
}

pub fn discover(app: &AppHandle) -> Vec<PetChoice> {
    let mut choices = HashMap::new();
    choices.insert(
        "spiderman4-sticker".to_string(),
        PetChoice {
            manifest_path: SPIDER_MAN_SELECTION.into(),
            id: "spiderman4-sticker".into(),
            display_name: "蜘蛛侠 4 · 吊线摆荡".into(),
            description:
                "The recent Spider-Man 4 limited sticker swings in from two overhead wires.".into(),
        },
    );
    extend_choices(&mut choices, &bundled_market_root(app));
    if let Ok(home) = app.path().home_dir() {
        extend_choices(&mut choices, &home.join(".codex").join("pets"));
    }
    let mut choices: Vec<_> = choices.into_values().collect();
    choices.sort_by(|left, right| left.display_name.cmp(&right.display_name));
    choices
}

fn extend_choices(choices: &mut HashMap<String, PetChoice>, root: &Path) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for choice in entries.filter_map(Result::ok).filter_map(|entry| {
        let path = entry.path().join("pet.json");
        let raw = fs::read_to_string(&path).ok()?;
        let manifest = parse_manifest(&raw).ok()?;
        let sprite = path.parent()?.join(&manifest.spritesheet_path);
        if !sprite.is_file() {
            return None;
        }
        (!matches!(manifest.id.as_str(), "spiderman4-sticker" | "webby")).then_some(PetChoice {
            manifest_path: path.to_string_lossy().into_owned(),
            id: manifest.id,
            display_name: manifest.display_name,
            description: manifest.description,
        })
    }) {
        choices.insert(choice.id.clone(), choice);
    }
}

pub(crate) fn bundled_market_root(app: &AppHandle) -> PathBuf {
    let installed = app
        .path()
        .resource_dir()
        .map(|directory| directory.join("market-pets"))
        .unwrap_or_default();
    if installed.is_dir() {
        installed
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("market-pets")
    }
}

pub(crate) fn bundled_manifest_path(app: &AppHandle, id: &str) -> Option<PathBuf> {
    let manifest = bundled_market_root(app).join(id).join("pet.json");
    manifest.is_file().then_some(manifest)
}

pub fn validate_manifest_path(path: &str) -> Result<String, String> {
    if path == SPIDER_MAN_SELECTION {
        spider_man_payload()?;
        return Ok(SPIDER_MAN_SELECTION.into());
    }
    let path = Path::new(path);
    if path.file_name().and_then(|value| value.to_str()) != Some("pet.json") {
        return Err("请选择 Codex Pet 目录中的 pet.json".into());
    }
    load_payload(path)?;
    path.canonicalize()
        .map(|value| value.to_string_lossy().into_owned())
        .map_err(|error| error.to_string())
}

pub(crate) fn parse_manifest(raw: &str) -> Result<CodexPetManifest, String> {
    let manifest: CodexPetManifest =
        serde_json::from_str(raw).map_err(|error| format!("pet.json 无效：{error}"))?;
    if manifest.id.trim().is_empty()
        || manifest.display_name.trim().is_empty()
        || manifest.spritesheet_path.trim().is_empty()
    {
        return Err("pet.json 缺少 id、displayName 或 spritesheetPath".into());
    }
    if Path::new(&manifest.spritesheet_path).is_absolute()
        || manifest
            .spritesheet_path
            .split(['/', '\\'])
            .any(|part| part == "..")
    {
        return Err("spritesheetPath 必须是宠物目录内的相对路径".into());
    }
    Ok(manifest)
}

pub(crate) fn validate_sprite_dimensions(image: &[u8]) -> Result<(), String> {
    atlas_geometry(image).map(|_| ())
}

fn representative_frame_data_url(image: &[u8]) -> Result<String, String> {
    let decoded =
        image::load_from_memory(image).map_err(|error| format!("无法生成 Pet 预览：{error}"))?;
    let (_, rows) = validate_atlas_dimensions(decoded.dimensions())?;
    let rgba = decoded.to_rgba8();
    let mut best = (0_u32, 0_u32, 0_u64);
    for row in 0..rows {
        for column in 0..8 {
            let x = column * 192;
            let y = row * 208;
            let visible = rgba
                .view(x, y, 192, 208)
                .pixels()
                .filter(|pixel| pixel.2[3] > 8)
                .count() as u64;
            if visible > best.2 {
                best = (x, y, visible);
            }
        }
    }
    if best.2 == 0 {
        return Err("序列帧图集没有可见内容".into());
    }
    let frame = image::imageops::crop_imm(&rgba, best.0, best.1, 192, 208).to_image();
    let mut encoded = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(frame)
        .write_to(&mut encoded, ImageFormat::Png)
        .map_err(|error| format!("生成 Pet 预览失败：{error}"))?;
    Ok(format!(
        "data:image/png;base64,{}",
        STANDARD.encode(encoded.into_inner())
    ))
}

fn payload(manifest: CodexPetManifest, image: &[u8], mime: &str) -> Result<PetPayload, String> {
    let (columns, rows) = atlas_geometry(image)?;
    Ok(PetPayload {
        manifest,
        spritesheet_data_url: format!("data:{mime};base64,{}", STANDARD.encode(image)),
        frame_width: 192,
        frame_height: 208,
        columns,
        rows,
    })
}

fn atlas_geometry(image: &[u8]) -> Result<(u32, u32), String> {
    let reader = ImageReader::new(Cursor::new(image))
        .with_guessed_format()
        .map_err(|error| format!("无法识别序列帧图集：{error}"))?;
    let dimensions = reader
        .into_dimensions()
        .map_err(|error| format!("无法读取序列帧图集尺寸：{error}"))?;
    validate_atlas_dimensions(dimensions)
}

fn validate_atlas_dimensions((width, height): (u32, u32)) -> Result<(u32, u32), String> {
    let columns = width / 192;
    let rows = height / 208;
    if width != 1536 || height % 208 != 0 || columns != 8 || !(9..=11).contains(&rows) {
        return Err(format!(
            "序列帧图集必须是 1536×1872（8×9）或兼容的 8×10/8×11 扩展图集，当前为 {width}×{height}"
        ));
    }
    Ok((columns, rows))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn bundled_pet_is_valid() {
        let pet = builtin_payload().unwrap();
        assert_eq!(pet.frame_width * pet.columns, 1536);
        assert_eq!(pet.frame_height * pet.rows, 1872);
        assert!(!pet.spritesheet_data_url.is_empty());
    }

    #[test]
    fn bundled_spider_man_is_valid_and_selectable() {
        let pet = selected_payload(Some(SPIDER_MAN_SELECTION)).unwrap();
        assert_eq!(pet.manifest.id, "spiderman4-sticker");
        assert_eq!(pet.frame_width * pet.columns, 1536);
        assert_eq!(pet.frame_height * pet.rows, 1872);
        assert_eq!(
            validate_manifest_path(SPIDER_MAN_SELECTION).unwrap(),
            SPIDER_MAN_SELECTION
        );
    }

    #[test]
    fn rejects_escaping_sprite_path() {
        let raw = r#"{
          "id":"bad",
          "displayName":"Bad",
          "description":"Bad path",
          "spritesheetPath":"../secret.png"
        }"#;
        assert!(parse_manifest(raw).is_err());
    }

    #[test]
    fn bundled_thumbnail_is_visible() {
        let thumbnail = selected_thumbnail(None).unwrap();
        assert!(thumbnail
            .image_data_url
            .starts_with("data:image/png;base64,"));
    }

    #[test]
    fn accepts_current_extended_codex_atlas_geometry() {
        assert_eq!(validate_atlas_dimensions((1536, 2288)).unwrap(), (8, 11));
        assert!(validate_atlas_dimensions((1536, 1664)).is_err());
    }

    #[test]
    fn bundled_market_contains_one_hundred_valid_unique_pets() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("market-pets");
        let mut ids = HashSet::new();
        let mut count = 0;
        for entry in fs::read_dir(root).unwrap().filter_map(Result::ok) {
            let manifest_path = entry.path().join("pet.json");
            let payload = load_payload(&manifest_path).unwrap_or_else(|error| {
                panic!("{}: {error}", manifest_path.display());
            });
            assert_eq!(
                payload.manifest.id,
                entry.file_name().to_string_lossy(),
                "folder id must match manifest id"
            );
            assert!(ids.insert(payload.manifest.id));
            assert_eq!(payload.columns, 8);
            assert!((9..=11).contains(&payload.rows));
            count += 1;
        }
        assert_eq!(count, 100);
    }
}
