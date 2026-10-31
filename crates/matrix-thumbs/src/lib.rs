// SPDX-License-Identifier: GPL-3.0-or-later
//! Thumbnail generation with a content-addressed on-disk cache.
//!
//! Cache keys include path, size, mtime and requested pixel size, so stale
//! thumbnails are never served after a file changes.

use image::{DynamicImage, ImageFormat, ImageReader};
use serde::Serialize;
use std::fs;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::UNIX_EPOCH;

#[derive(Debug, thiserror::Error)]
pub enum ThumbError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("decode: {0}")]
    Decode(#[from] image::ImageError),
    #[error("unsupported file type")]
    Unsupported,
    #[error("{0}")]
    External(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ThumbKind {
    Image,
    Video,
    Pdf,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbRef {
    pub path: String,
    pub thumb_path: String,
    pub width: u32,
    pub height: u32,
    pub cached: bool,
    pub kind: ThumbKind,
}

pub struct Thumbnailer {
    cache_dir: PathBuf,
    ffmpeg: bool,
    pdftoppm: bool,
}

const IMAGE_EXT: &[&str] = &[
    "jpg", "jpeg", "png", "gif", "webp", "bmp", "tif", "tiff", "ico", "avif", "heic", "heif",
    "qoi", "tga", "dds", "exr", "hdr", "pnm",
];
const VIDEO_EXT: &[&str] = &[
    "mp4", "m4v", "mkv", "webm", "mov", "avi", "wmv", "flv", "mpg", "mpeg", "3gp", "ts",
];

fn ext_of(path: &Path) -> String {
    path.extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default()
}

pub fn kind_for(path: &Path) -> Option<ThumbKind> {
    let ext = ext_of(path);
    if IMAGE_EXT.contains(&ext.as_str()) {
        Some(ThumbKind::Image)
    } else if VIDEO_EXT.contains(&ext.as_str()) {
        Some(ThumbKind::Video)
    } else if ext == "pdf" {
        Some(ThumbKind::Pdf)
    } else {
        None
    }
}

fn has_binary(name: &str) -> bool {
    Command::new(name)
        .arg("-version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn exif_orientation(path: &Path) -> u32 {
    let Ok(file) = fs::File::open(path) else {
        return 1;
    };
    let mut reader = BufReader::new(file);
    let Ok(exif) = exif::Reader::new().read_from_container(&mut reader) else {
        return 1;
    };
    exif.get_field(exif::Tag::Orientation, exif::In::PRIMARY)
        .and_then(|f| f.value.get_uint(0))
        .unwrap_or(1)
}

fn apply_orientation(img: DynamicImage, orientation: u32) -> DynamicImage {
    match orientation {
        2 => img.fliph(),
        3 => img.rotate180(),
        4 => img.flipv(),
        5 => img.rotate90().fliph(),
        6 => img.rotate90(),
        7 => img.rotate270().fliph(),
        8 => img.rotate270(),
        _ => img,
    }
}

impl Thumbnailer {
    pub fn new(cache_dir: PathBuf) -> Self {
        let _ = fs::create_dir_all(&cache_dir);
        Self {
            cache_dir,
            ffmpeg: has_binary("ffmpeg"),
            pdftoppm: has_binary("pdftoppm"),
        }
    }

    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Generate (or reuse) a thumbnail for `path`, fitting within `px` pixels.
    pub fn thumbnail(&self, path: &Path, px: u32) -> Result<ThumbRef, ThumbError> {
        let px = px.clamp(32, 1024);
        let kind = kind_for(path).ok_or(ThumbError::Unsupported)?;

        let meta = fs::metadata(path)?;
        let mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_millis())
            .unwrap_or(0);

        let key = format!("{}|{}|{}|{}", path.display(), meta.len(), mtime, px);
        let hash = blake3::hash(key.as_bytes()).to_hex().to_string();
        let out_dir = self.cache_dir.join(&hash[..2]);
        fs::create_dir_all(&out_dir)?;
        let out = out_dir.join(format!("{hash}.webp"));

        if out.exists() {
            let (width, height) = image::image_dimensions(&out).unwrap_or((px, px));
            return Ok(ThumbRef {
                path: path.to_string_lossy().into_owned(),
                thumb_path: out.to_string_lossy().into_owned(),
                width,
                height,
                cached: true,
                kind,
            });
        }

        let img = self.render(path, kind, px)?;
        let img = apply_orientation(
            img,
            if kind == ThumbKind::Image {
                exif_orientation(path)
            } else {
                1
            },
        );
        let thumb = img.thumbnail(px, px);

        let tmp = out_dir.join(format!("{hash}.tmp.webp"));
        let _ = fs::remove_file(&tmp);
        thumb.save_with_format(&tmp, ImageFormat::WebP)?;
        fs::rename(&tmp, &out)?;

        Ok(ThumbRef {
            path: path.to_string_lossy().into_owned(),
            thumb_path: out.to_string_lossy().into_owned(),
            width: thumb.width(),
            height: thumb.height(),
            cached: false,
            kind,
        })
    }

    fn render(&self, path: &Path, kind: ThumbKind, px: u32) -> Result<DynamicImage, ThumbError> {
        match kind {
            ThumbKind::Image => {
                let reader = ImageReader::open(path)?.with_guessed_format()?;
                Ok(reader.decode()?)
            }
            ThumbKind::Video => {
                if !self.ffmpeg {
                    return Err(ThumbError::External("ffmpeg not installed".into()));
                }
                let frame = std::env::temp_dir().join(format!(
                    "matrix-thumb-{}-{}.png",
                    std::process::id(),
                    blake3::hash(path.to_string_lossy().as_bytes()).to_hex()
                ));
                let _ = fs::remove_file(&frame);
                let status = Command::new("ffmpeg")
                    .args(["-hide_banner", "-loglevel", "error", "-y", "-ss", "1", "-i"])
                    .arg(path)
                    .args(["-frames:v", "1", "-vf", &format!("scale={px}:-1")])
                    .arg(&frame)
                    .status()?;
                if !status.success() || !frame.exists() {
                    return Err(ThumbError::External(
                        "ffmpeg could not extract a frame".into(),
                    ));
                }
                let img = ImageReader::open(&frame)?.with_guessed_format()?.decode()?;
                let _ = fs::remove_file(&frame);
                Ok(img)
            }
            ThumbKind::Pdf => {
                if !self.pdftoppm {
                    return Err(ThumbError::External("pdftoppm not installed".into()));
                }
                let prefix = std::env::temp_dir().join(format!(
                    "matrix-pdf-{}-{}",
                    std::process::id(),
                    blake3::hash(path.to_string_lossy().as_bytes()).to_hex()
                ));
                let status = Command::new("pdftoppm")
                    .args([
                        "-f",
                        "1",
                        "-singlefile",
                        "-scale-to",
                        &px.to_string(),
                        "-png",
                    ])
                    .arg(path)
                    .arg(&prefix)
                    .status()?;
                let page = prefix.with_extension("png");
                if !status.success() || !page.exists() {
                    return Err(ThumbError::External(
                        "pdftoppm could not render page 1".into(),
                    ));
                }
                let img = ImageReader::open(&page)?.with_guessed_format()?.decode()?;
                let _ = fs::remove_file(&page);
                Ok(img)
            }
        }
    }
}
