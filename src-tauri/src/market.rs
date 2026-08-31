use reqwest::Client;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::HashSet,
    fs,
    io::{Cursor, Read},
    path::Path,
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager};

use crate::pet;

const MARKET_ORIGIN: &str = "https://codex-pets.net";
const PAGE_SIZE: u32 = 30;
const MARKET_LIMIT: usize = 100;
const MAX_DOWNLOAD_BYTES: u64 = 30 * 1024 * 1024;
const MAX_MANIFEST_BYTES: u64 = 64 * 1024;
const MAX_SPRITESHEET_BYTES: u64 = 28 * 1024 * 1024;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RemotePet {
    id: String,
    display_name: String,
    description: String,
    #[serde(default)]
    owner_name: Option<String>,
    #[serde(default)]
    owner_handle: Option<String>,
    like_count: u64,
    poster_url: String,
    preview_url: String,
    download_url: String,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PetPage {
    pets: Vec<RemotePet>,
    total_pages: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketPet {
    pub rank: usize,
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub creator: String,
    pub like_count: u64,
    pub poster_url: String,
    pub preview_url: String,
    pub tags: Vec<String>,
    pub installed: bool,
    pub manifest_path: Option<String>,
    download_url: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketCatalog {
    pub pets: Vec<MarketPet>,
    pub source: &'static str,
    pub warning: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketInstallResult {
    pub id: String,
    pub manifest_path: String,
    pub already_installed: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MarketInstallSummary {
    pub installed: usize,
    pub skipped: usize,
    pub failed: usize,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MarketInstallProgress {
    current: usize,
    total: usize,
    id: String,
    display_name: String,
    status: &'static str,
    error: Option<String>,
}

pub async fn list(app: &AppHandle, refresh: bool) -> Result<MarketCatalog, String> {
    if !refresh {
        if let Ok((pets, source)) = load_market_fallback(app) {
            return Ok(MarketCatalog {
                pets,
                source,
                warning: None,
            });
        }
    }
    match fetch_market_catalog(app).await {
        Ok(pets) => {
            save_market_cache(app, &pets)?;
            Ok(MarketCatalog {
                pets,
                source: "live",
                warning: None,
            })
        }
        Err(error) => {
            let (pets, source) = load_market_fallback(app).map_err(|cache_error| {
                format!("{error}；本地也没有可用的市场缓存：{cache_error}")
            })?;
            Ok(MarketCatalog {
                pets,
                source,
                warning: Some(format!("官网暂时不可达，正在显示最近缓存：{error}")),
            })
        }
    }
}

pub async fn install(app: &AppHandle, id: &str) -> Result<MarketInstallResult, String> {
    validate_pet_id(id)?;
    let pet = catalog_pets(app)
        .await?
        .into_iter()
        .find(|pet| pet.id == id)
        .ok_or_else(|| "该宠物不在已过滤的 Liked Top 100 中".to_string())?;
    install_catalog_pet(app, &pet).await
}

pub async fn install_all(app: &AppHandle) -> Result<MarketInstallSummary, String> {
    let pets = catalog_pets(app).await?;
    let total = pets.len();
    let mut summary = MarketInstallSummary {
        installed: 0,
        skipped: 0,
        failed: 0,
    };

    for (index, pet) in pets.into_iter().enumerate() {
        let current = index + 1;
        let result = install_catalog_pet(app, &pet).await;
        match result {
            Ok(result) => {
                if result.already_installed {
                    summary.skipped += 1;
                } else {
                    summary.installed += 1;
                }
                let _ = app.emit(
                    "market-install-progress",
                    MarketInstallProgress {
                        current,
                        total,
                        id: pet.id,
                        display_name: pet.display_name,
                        status: if result.already_installed {
                            "skipped"
                        } else {
                            "installed"
                        },
                        error: None,
                    },
                );
            }
            Err(error) => {
                summary.failed += 1;
                let _ = app.emit(
                    "market-install-progress",
                    MarketInstallProgress {
                        current,
                        total,
                        id: pet.id,
                        display_name: pet.display_name,
                        status: "failed",
                        error: Some(error),
                    },
                );
            }
        }
    }
    Ok(summary)
}

pub fn import_zip(app: &AppHandle, archive_path: &str) -> Result<MarketInstallResult, String> {
    let archive_path = Path::new(archive_path);
    if archive_path
        .extension()
        .and_then(|value| value.to_str())
        .is_none_or(|extension| !extension.eq_ignore_ascii_case("zip"))
    {
        return Err("请选择 .zip 格式的 Codex Pet 安装包".into());
    }
    let metadata = fs::metadata(archive_path).map_err(|error| format!("读取 ZIP 失败：{error}"))?;
    if metadata.len() > MAX_DOWNLOAD_BYTES {
        return Err("Pet ZIP 不能超过 30 MB".into());
    }
    let bytes = fs::read(archive_path).map_err(|error| format!("读取 ZIP 失败：{error}"))?;
    let manifest = archive_manifest(&bytes)?;
    validate_pet_id(&manifest.id)?;
    let home = app.path().home_dir().map_err(|error| error.to_string())?;
    let already_installed = home
        .join(".codex")
        .join("pets")
        .join(&manifest.id)
        .join("pet.json")
        .is_file();
    let manifest_path = install_archive(&home, &manifest.id, &bytes)?;
    Ok(MarketInstallResult {
        id: manifest.id,
        manifest_path,
        already_installed,
    })
}

async fn catalog_pets(app: &AppHandle) -> Result<Vec<MarketPet>, String> {
    if let Ok((pets, _)) = load_market_fallback(app) {
        return Ok(pets);
    }
    match fetch_market_catalog(app).await {
        Ok(pets) => {
            save_market_cache(app, &pets)?;
            Ok(pets)
        }
        Err(error) => load_market_fallback(app)
            .map(|(pets, _)| pets)
            .map_err(|cache_error| format!("{error}；市场缓存不可用：{cache_error}")),
    }
}

async fn fetch_market_catalog(app: &AppHandle) -> Result<Vec<MarketPet>, String> {
    let client = market_client()?;
    let leader_ids = fetch_leader_ids(&client).await?;
    let mut seen = HashSet::new();
    let mut market = Vec::with_capacity(MARKET_LIMIT);
    let mut page = 1;

    while market.len() < MARKET_LIMIT {
        let response: PetPage = fetch_json(
            &client,
            &format!("/api/pets?page={page}&pageSize={PAGE_SIZE}&sort=popular"),
        )
        .await?;
        let total_pages = response.total_pages;
        for remote in response.pets {
            if leader_ids.contains(&remote.id)
                || has_leader_tag(&remote.tags)
                || !seen.insert(remote.id.clone())
            {
                continue;
            }
            let manifest_path = installed_manifest(app, &remote.id);
            market.push(MarketPet {
                rank: market.len() + 1,
                id: remote.id,
                display_name: remote.display_name,
                description: remote.description,
                creator: remote
                    .owner_name
                    .or(remote.owner_handle)
                    .unwrap_or_else(|| "Anonymous".into()),
                like_count: remote.like_count,
                poster_url: remote.poster_url,
                preview_url: remote.preview_url,
                tags: remote.tags,
                installed: manifest_path.is_some(),
                manifest_path,
                download_url: remote.download_url,
            });
            if market.len() == MARKET_LIMIT {
                break;
            }
        }
        if page >= total_pages || total_pages == 0 {
            break;
        }
        page += 1;
    }

    if market.len() < MARKET_LIMIT {
        return Err(format!("过滤 Leaders 后只获得 {} 个市场宠物", market.len()));
    }
    Ok(market)
}

async fn fetch_leader_ids(client: &Client) -> Result<HashSet<String>, String> {
    let mut ids = HashSet::new();
    let mut page = 1;
    loop {
        let response: PetPage = fetch_json(
            client,
            &format!("/api/collections/leaders?page={page}&pageSize={PAGE_SIZE}"),
        )
        .await?;
        let total_pages = response.total_pages;
        ids.extend(response.pets.into_iter().map(|pet| pet.id));
        if page >= total_pages || total_pages == 0 {
            break;
        }
        page += 1;
    }
    Ok(ids)
}

fn market_client() -> Result<Client, String> {
    Client::builder()
        .user_agent("Con-Pet/0.1 (+https://codex-pets.net)")
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(45))
        .build()
        .map_err(|error| format!("市场网络初始化失败：{error}"))
}

async fn fetch_json<T: DeserializeOwned>(client: &Client, path: &str) -> Result<T, String> {
    client
        .get(format!("{MARKET_ORIGIN}{path}"))
        .header("accept", "application/json")
        .send()
        .await
        .map_err(|error| format!("市场请求失败：{error}"))?
        .error_for_status()
        .map_err(|error| format!("市场返回错误：{error}"))?
        .json::<T>()
        .await
        .map_err(|error| format!("市场数据无效：{error}"))
}

async fn install_catalog_pet(
    app: &AppHandle,
    market_pet: &MarketPet,
) -> Result<MarketInstallResult, String> {
    if let Some(path) = installed_manifest(app, &market_pet.id) {
        return Ok(MarketInstallResult {
            id: market_pet.id.clone(),
            manifest_path: path,
            already_installed: true,
        });
    }
    validate_pet_id(&market_pet.id)?;
    let url = if market_pet.download_url.starts_with('/') {
        format!("{MARKET_ORIGIN}{}", market_pet.download_url)
    } else {
        return Err("市场下载地址无效".into());
    };
    let response = market_client()?
        .get(url)
        .header("accept", "application/zip")
        .send()
        .await
        .map_err(|error| format!("下载 {} 失败：{error}", market_pet.display_name))?
        .error_for_status()
        .map_err(|error| format!("下载 {} 失败：{error}", market_pet.display_name))?;
    if response
        .content_length()
        .is_some_and(|length| length > MAX_DOWNLOAD_BYTES)
    {
        return Err(format!("{} 的安装包超过 30 MB", market_pet.display_name));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|error| format!("读取 {} 安装包失败：{error}", market_pet.display_name))?;
    if bytes.len() as u64 > MAX_DOWNLOAD_BYTES {
        return Err(format!("{} 的安装包超过 30 MB", market_pet.display_name));
    }

    let home = app.path().home_dir().map_err(|error| error.to_string())?;
    let id = market_pet.id.clone();
    let manifest_path =
        tauri::async_runtime::spawn_blocking(move || install_archive(&home, &id, bytes.as_ref()))
            .await
            .map_err(|error| format!("安装任务失败：{error}"))??;

    Ok(MarketInstallResult {
        id: market_pet.id.clone(),
        manifest_path,
        already_installed: false,
    })
}

fn install_archive(home: &Path, expected_id: &str, bytes: &[u8]) -> Result<String, String> {
    let root = home.join(".codex").join("pets");
    let final_dir = root.join(expected_id);
    let final_manifest = final_dir.join("pet.json");
    if final_manifest.is_file() {
        pet::load_payload(&final_manifest)?;
        return canonical_string(&final_manifest);
    }

    fs::create_dir_all(&root).map_err(|error| format!("创建宠物目录失败：{error}"))?;
    let staging = root.join(format!(
        ".pc-pet-install-{expected_id}-{}",
        std::process::id()
    ));
    if staging.exists() {
        fs::remove_dir_all(&staging).map_err(|error| format!("清理安装缓存失败：{error}"))?;
    }
    fs::create_dir_all(&staging).map_err(|error| format!("创建安装缓存失败：{error}"))?;

    let result = (|| {
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
            .map_err(|error| format!("安装包不是有效 ZIP：{error}"))?;
        if archive.len() > 16 {
            return Err("安装包文件数量异常".into());
        }
        let manifest_raw = {
            let mut entry = archive
                .by_name("pet.json")
                .map_err(|_| "安装包缺少 pet.json".to_string())?;
            if entry.size() > MAX_MANIFEST_BYTES {
                return Err("pet.json 过大".into());
            }
            let mut raw = String::new();
            entry
                .read_to_string(&mut raw)
                .map_err(|error| format!("读取 pet.json 失败：{error}"))?;
            raw
        };
        let manifest = pet::parse_manifest(&manifest_raw)?;
        if manifest.id != expected_id {
            return Err("安装包 ID 与市场条目不一致".into());
        }
        let extension = Path::new(&manifest.spritesheet_path)
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        if !matches!(extension.as_str(), "png" | "webp") {
            return Err("市场宠物图集必须是 PNG 或 WebP".into());
        }
        let spritesheet = {
            let mut entry = archive
                .by_name(&manifest.spritesheet_path)
                .map_err(|_| "安装包缺少序列帧图集".to_string())?;
            if entry.size() > MAX_SPRITESHEET_BYTES {
                return Err("序列帧图集超过 28 MB".into());
            }
            let mut data = Vec::with_capacity(entry.size() as usize);
            entry
                .read_to_end(&mut data)
                .map_err(|error| format!("读取序列帧图集失败：{error}"))?;
            data
        };
        pet::validate_sprite_dimensions(&spritesheet)?;
        let staged_manifest = staging.join("pet.json");
        let staged_sprite = staging.join(&manifest.spritesheet_path);
        if !staged_sprite.starts_with(&staging) {
            return Err("序列帧路径越界".into());
        }
        if let Some(parent) = staged_sprite.parent() {
            fs::create_dir_all(parent).map_err(|error| format!("创建图集目录失败：{error}"))?;
        }
        fs::write(&staged_manifest, manifest_raw)
            .map_err(|error| format!("写入 pet.json 失败：{error}"))?;
        fs::write(&staged_sprite, spritesheet)
            .map_err(|error| format!("写入序列帧图集失败：{error}"))?;
        pet::load_payload(&staged_manifest)?;
        fs::rename(&staging, &final_dir).map_err(|error| format!("完成安装失败：{error}"))?;
        canonical_string(&final_manifest)
    })();

    if result.is_err() {
        let _ = fs::remove_dir_all(&staging);
    }
    result
}

fn archive_manifest(bytes: &[u8]) -> Result<pet::CodexPetManifest, String> {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes))
        .map_err(|error| format!("安装包不是有效 ZIP：{error}"))?;
    if archive.len() > 16 {
        return Err("安装包文件数量异常".into());
    }
    let mut entry = archive
        .by_name("pet.json")
        .map_err(|_| "安装包根目录缺少 pet.json".to_string())?;
    if entry.size() > MAX_MANIFEST_BYTES {
        return Err("pet.json 过大".into());
    }
    let mut raw = String::new();
    entry
        .read_to_string(&mut raw)
        .map_err(|error| format!("读取 pet.json 失败：{error}"))?;
    pet::parse_manifest(&raw)
}

fn market_cache_path(app: &AppHandle) -> Result<std::path::PathBuf, String> {
    app.path()
        .app_cache_dir()
        .map(|directory| directory.join("market-liked-top100.json"))
        .map_err(|error| error.to_string())
}

fn save_market_cache(app: &AppHandle, pets: &[MarketPet]) -> Result<(), String> {
    let path = market_cache_path(app)?;
    let parent = path
        .parent()
        .ok_or_else(|| "市场缓存目录无效".to_string())?;
    fs::create_dir_all(parent).map_err(|error| format!("创建市场缓存失败：{error}"))?;
    let data = serde_json::to_vec(pets).map_err(|error| format!("序列化市场缓存失败：{error}"))?;
    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, data).map_err(|error| format!("写入市场缓存失败：{error}"))?;
    fs::rename(&temporary, path).map_err(|error| format!("保存市场缓存失败：{error}"))
}

fn load_market_fallback(app: &AppHandle) -> Result<(Vec<MarketPet>, &'static str), String> {
    let user_cache = market_cache_path(app)?;
    if let Ok(pets) = load_market_cache_file(app, &user_cache) {
        return Ok((pets, "cache"));
    }
    let bundled_cache = app
        .path()
        .resource_dir()
        .map(|directory| directory.join("market-catalog.json"))
        .unwrap_or_default();
    let bundled_cache = if bundled_cache.is_file() {
        bundled_cache
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("market-catalog.json")
    };
    load_market_cache_file(app, &bundled_cache).map(|pets| (pets, "bundled"))
}

fn load_market_cache_file(app: &AppHandle, path: &Path) -> Result<Vec<MarketPet>, String> {
    let raw = fs::read(path).map_err(|error| format!("读取市场缓存失败：{error}"))?;
    let mut pets: Vec<MarketPet> =
        serde_json::from_slice(&raw).map_err(|error| format!("市场缓存无效：{error}"))?;
    if pets.len() != MARKET_LIMIT || pets.iter().any(|pet| has_leader_tag(&pet.tags)) {
        return Err("市场缓存不完整或包含 Leaders".into());
    }
    for (index, market_pet) in pets.iter_mut().enumerate() {
        market_pet.rank = index + 1;
        market_pet.manifest_path = installed_manifest(app, &market_pet.id);
        market_pet.installed = market_pet.manifest_path.is_some();
    }
    Ok(pets)
}

fn installed_manifest(app: &AppHandle, id: &str) -> Option<String> {
    let home = app.path().home_dir().ok()?;
    let manifest = home.join(".codex").join("pets").join(id).join("pet.json");
    if manifest.is_file() {
        return canonical_string(&manifest).ok();
    }
    pet::bundled_manifest_path(app, id).and_then(|path| canonical_string(&path).ok())
}

fn canonical_string(path: &Path) -> Result<String, String> {
    path.canonicalize()
        .map(|path| path.to_string_lossy().into_owned())
        .map_err(|error| error.to_string())
}

fn has_leader_tag(tags: &[String]) -> bool {
    tags.iter()
        .any(|tag| tag.eq_ignore_ascii_case("leader") || tag.eq_ignore_ascii_case("leaders"))
}

fn validate_pet_id(id: &str) -> Result<(), String> {
    if id.is_empty()
        || id.len() > 128
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err("市场宠物 ID 无效".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, ImageFormat, Rgba, RgbaImage};
    use std::io::Write;
    use zip::{write::SimpleFileOptions, ZipWriter};

    #[test]
    fn rejects_unsafe_market_ids() {
        assert!(validate_pet_id("cute-pet_2").is_ok());
        assert!(validate_pet_id("../escape").is_err());
        assert!(validate_pet_id("pet/name").is_err());
    }

    #[test]
    fn recognizes_singular_and_plural_leader_tags() {
        assert!(has_leader_tag(&["cute".into(), "Leaders".into()]));
        assert!(has_leader_tag(&["leader".into()]));
        assert!(!has_leader_tag(&["celeb".into()]));
    }

    #[test]
    fn imports_a_valid_flat_pet_zip() {
        let mut image = RgbaImage::new(1536, 1872);
        image.put_pixel(20, 20, Rgba([80, 220, 180, 255]));
        let mut png = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(image)
            .write_to(&mut png, ImageFormat::Png)
            .unwrap();

        let manifest = r#"{
          "id":"pc-pet-import-test",
          "displayName":"Import Test",
          "description":"Safe ZIP fixture",
          "spritesheetPath":"spritesheet.png"
        }"#;
        let cursor = Cursor::new(Vec::new());
        let mut writer = ZipWriter::new(cursor);
        writer
            .start_file("pet.json", SimpleFileOptions::default())
            .unwrap();
        writer.write_all(manifest.as_bytes()).unwrap();
        writer
            .start_file("spritesheet.png", SimpleFileOptions::default())
            .unwrap();
        writer.write_all(png.get_ref()).unwrap();
        let bytes = writer.finish().unwrap().into_inner();

        let root = std::env::temp_dir().join(format!(
            "pc-pet-import-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).unwrap();
        let installed = install_archive(&root, "pc-pet-import-test", &bytes).unwrap();
        assert!(Path::new(&installed).is_file());
        assert!(root
            .join(".codex/pets/pc-pet-import-test/spritesheet.png")
            .is_file());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn bundled_catalog_has_one_hundred_non_leader_pets() {
        let pets: Vec<MarketPet> =
            serde_json::from_str(include_str!("../resources/market-catalog.json")).unwrap();
        assert_eq!(pets.len(), 100);
        assert!(pets.iter().all(|pet| !has_leader_tag(&pet.tags)));
        let ids: HashSet<_> = pets.iter().map(|pet| &pet.id).collect();
        assert_eq!(ids.len(), 100);
    }
}
