//! The picture a viewer puts behind their own map: where it goes on disk, what counts as
//! an image, and how big it may be. The row in `map_user_settings` keeps the relative
//! path; this module owns everything under the uploads directory.

use std::path::{Path, PathBuf};

use sqlx::PgPool;

use super::MapError;

/// The largest image accepted. Bigger than any sensible backdrop; the limit is there so a
/// viewer cannot fill the disk.
pub const MAX_IMAGE_BYTES: usize = 8 * 1024 * 1024;

super::text_enum! {
    /// How the image sits under the map. `Grid` spans the whole world and moves with the
    /// systems; `Viewport` fills the visible panel and stays put.
    pub enum BackgroundMode as "map_background_mode" {
        Grid => "grid",
        Viewport => "viewport",
    }
}

/// An image format told apart by its leading bytes, which is the only thing checked: a
/// browser's declared content type is a hint, the file is the truth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    Gif,
    Webp,
}

impl ImageFormat {
    pub fn sniff(bytes: &[u8]) -> Option<ImageFormat> {
        if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            Some(ImageFormat::Png)
        } else if bytes.starts_with(b"\xff\xd8\xff") {
            Some(ImageFormat::Jpeg)
        } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
            Some(ImageFormat::Gif)
        } else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
            Some(ImageFormat::Webp)
        } else {
            None
        }
    }

    pub fn from_extension(ext: &str) -> Option<ImageFormat> {
        match ext {
            "png" => Some(ImageFormat::Png),
            "jpg" => Some(ImageFormat::Jpeg),
            "gif" => Some(ImageFormat::Gif),
            "webp" => Some(ImageFormat::Webp),
            _ => None,
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
            ImageFormat::Gif => "gif",
            ImageFormat::Webp => "webp",
        }
    }

    pub fn content_type(self) -> &'static str {
        match self {
            ImageFormat::Png => "image/png",
            ImageFormat::Jpeg => "image/jpeg",
            ImageFormat::Gif => "image/gif",
            ImageFormat::Webp => "image/webp",
        }
    }
}

/// Check that `bytes` is an image this feature accepts, and say which.
pub fn validate(bytes: &[u8]) -> super::Result<ImageFormat> {
    if bytes.len() > MAX_IMAGE_BYTES {
        return Err(MapError::Validation(format!(
            "the image is too large: {} MiB is the most it can be",
            MAX_IMAGE_BYTES / 1024 / 1024
        )));
    }
    ImageFormat::sniff(bytes)
        .ok_or_else(|| MapError::Validation("that is not a PNG, JPEG, GIF or WebP image".into()))
}

/// The directory the uploads live in. Cloned into the app state; a path is all it holds.
#[derive(Debug, Clone)]
pub struct BackgroundStore {
    root: PathBuf,
}

impl BackgroundStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        BackgroundStore { root: root.into() }
    }

    /// Store `bytes` as this user's background on this map, and hand back the relative
    /// path the settings row keeps. The name carries a timestamp so a replacement is a
    /// new URL and the browser fetches it rather than showing the old one from cache.
    pub async fn save(&self, map_id: i64, user_id: i64, bytes: &[u8]) -> super::Result<String> {
        let format = validate(bytes)?;
        let stamp = chrono::Utc::now().timestamp_millis();
        let relative = format!(
            "map-backgrounds/{map_id}/{user_id}-{stamp}.{}",
            format.extension()
        );
        let path = self.root.join(&relative);
        if let Some(dir) = path.parent() {
            tokio::fs::create_dir_all(dir).await.map_err(io_error)?;
        }
        tokio::fs::write(&path, bytes).await.map_err(io_error)?;
        Ok(relative)
    }

    /// Read a stored image back, with the content type its extension says. `None` when
    /// the row points at a file that is no longer there.
    pub async fn read(&self, relative: &str) -> super::Result<Option<(ImageFormat, Vec<u8>)>> {
        let Some(format) = format_of(relative) else {
            return Ok(None);
        };
        match tokio::fs::read(self.root.join(relative)).await {
            Ok(bytes) => Ok(Some((format, bytes))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(io_error(e)),
        }
    }

    /// Remove a stored image. A file already gone is not an error: the row is what is
    /// being cleared, and the disk agreeing early is fine.
    pub async fn remove(&self, relative: &str) -> super::Result<()> {
        match tokio::fs::remove_file(self.root.join(relative)).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(io_error(e)),
        }
    }

    /// This user's image on this map, if they have one and it is still on disk.
    pub async fn image_for(
        &self,
        pool: &PgPool,
        map_id: i64,
        user_id: i64,
    ) -> super::Result<Option<(ImageFormat, Vec<u8>)>> {
        let path = sqlx::query_scalar!(
            "select background_image_path from map_user_settings
             where map_id = $1 and user_id = $2",
            map_id,
            user_id,
        )
        .fetch_optional(pool)
        .await?
        .flatten();
        match path {
            Some(path) => self.read(&path).await,
            None => Ok(None),
        }
    }

    /// Make `bytes` this user's image on this map. The new file is written before the row
    /// points at it, and the old one removed only after, so a failure halfway leaves a
    /// stray file rather than a row pointing at nothing.
    pub async fn replace(
        &self,
        pool: &PgPool,
        map_id: i64,
        user_id: i64,
        bytes: &[u8],
    ) -> super::Result<()> {
        let stored = self.save(map_id, user_id, bytes).await?;
        let previous = point_at(pool, map_id, user_id, Some(&stored)).await?;
        if let Some(old) = previous.filter(|old| old != &stored) {
            self.remove(&old).await?;
        }
        Ok(())
    }

    /// Take this user's image off this map, file and all.
    pub async fn clear(&self, pool: &PgPool, map_id: i64, user_id: i64) -> super::Result<()> {
        if let Some(old) = point_at(pool, map_id, user_id, None).await? {
            self.remove(&old).await?;
        }
        Ok(())
    }
}

/// Point the settings row at `path` (or at nothing) and hand back what it pointed at
/// before, so the caller can clear the old file once the row no longer refers to it.
async fn point_at(
    pool: &PgPool,
    map_id: i64,
    user_id: i64,
    path: Option<&str>,
) -> super::Result<Option<String>> {
    let mut tx = pool.begin().await?;
    let previous = sqlx::query_scalar!(
        "select background_image_path from map_user_settings
         where map_id = $1 and user_id = $2 for update",
        map_id,
        user_id,
    )
    .fetch_optional(&mut *tx)
    .await?
    .flatten();
    sqlx::query!(
        "insert into map_user_settings (map_id, user_id, background_image_path)
         values ($1, $2, $3)
         on conflict (map_id, user_id) do update set
             background_image_path = $3,
             updated_at = now()",
        map_id,
        user_id,
        path,
    )
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(previous)
}

fn format_of(relative: &str) -> Option<ImageFormat> {
    Path::new(relative)
        .extension()
        .and_then(|e| e.to_str())
        .and_then(ImageFormat::from_extension)
}

fn io_error(e: std::io::Error) -> MapError {
    MapError::Validation(format!("could not store the image: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const PNG: &[u8] = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR";

    fn scratch() -> BackgroundStore {
        let dir = std::env::temp_dir().join(format!(
            "ws-backgrounds-{}-{}",
            std::process::id(),
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        BackgroundStore::new(dir)
    }

    #[test]
    fn the_bytes_decide_the_format_not_the_name() {
        assert_eq!(ImageFormat::sniff(PNG), Some(ImageFormat::Png));
        assert_eq!(
            ImageFormat::sniff(b"\xff\xd8\xff\xe0JFIF"),
            Some(ImageFormat::Jpeg)
        );
        assert_eq!(ImageFormat::sniff(b"GIF89a....."), Some(ImageFormat::Gif));
        assert_eq!(
            ImageFormat::sniff(b"RIFF\0\0\0\0WEBPVP8 "),
            Some(ImageFormat::Webp)
        );
        assert_eq!(ImageFormat::sniff(b"<svg xmlns="), None);
        assert_eq!(ImageFormat::sniff(b""), None);
    }

    #[test]
    fn a_non_image_or_an_oversized_one_is_refused() {
        assert!(matches!(validate(b"hello"), Err(MapError::Validation(_))));
        let mut huge = PNG.to_vec();
        huge.resize(MAX_IMAGE_BYTES + 1, 0);
        assert!(matches!(validate(&huge), Err(MapError::Validation(_))));
        assert_eq!(validate(PNG).unwrap(), ImageFormat::Png);
    }

    #[tokio::test]
    async fn a_saved_image_reads_back_and_a_removed_one_is_gone() {
        let store = scratch();
        let relative = store.save(7, 3, PNG).await.unwrap();
        assert!(relative.starts_with("map-backgrounds/7/3-"));
        assert!(relative.ends_with(".png"));

        let (format, bytes) = store.read(&relative).await.unwrap().unwrap();
        assert_eq!(format, ImageFormat::Png);
        assert_eq!(bytes, PNG);

        store.remove(&relative).await.unwrap();
        assert!(store.read(&relative).await.unwrap().is_none());
        store.remove(&relative).await.unwrap();
        std::fs::remove_dir_all(&store.root).ok();
    }

    #[tokio::test]
    async fn a_file_that_is_not_an_image_never_reaches_the_disk() {
        let store = scratch();
        assert!(store.save(1, 1, b"not an image").await.is_err());
        assert!(!store.root.exists());
    }
}
