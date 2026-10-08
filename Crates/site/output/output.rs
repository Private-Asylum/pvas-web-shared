//! A site's files, and writing them to disk.

use std::borrow::Cow;
use std::fs;
use std::io;
use std::path::{Component, Path};

use pvas_web_assets::{Asset, YETI};

/// One file of a site: a rendered page or a static file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteFile {
    /// Where it is written, relative to the output root, with `/` separators: `index.html`,
    /// `css/site.css`. Never absolute and never climbing out with `..`; [`write()`] refuses both.
    pub path: String,
    /// The file's contents.
    pub bytes: Cow<'static, [u8]>,
}

impl SiteFile {
    /// A rendered page.
    #[must_use]
    pub fn page(path: impl Into<String>, html: String) -> Self {
        Self {
            path: path.into(),
            bytes: Cow::Owned(html.into_bytes()),
        }
    }

    /// An embedded asset, at the path it declares.
    #[must_use]
    pub fn asset(asset: &Asset) -> Self {
        Self {
            path: asset.path.to_owned(),
            bytes: Cow::Borrowed(asset.bytes),
        }
    }

    /// Every file under `dir`, recursively, placed under `prefix` in the output (`""` for the
    /// root). For a site's own stylesheets and static files, read at build time.
    ///
    /// # Errors
    ///
    /// Any error reading the directory or a file in it.
    pub fn dir(dir: &Path, prefix: &str) -> io::Result<Vec<Self>> {
        let mut files = Vec::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let path = if prefix.is_empty() {
                name
            } else {
                format!("{prefix}/{name}")
            };
            if entry.file_type()?.is_dir() {
                files.extend(Self::dir(&entry.path(), &path)?);
            } else {
                files.push(Self {
                    path,
                    bytes: Cow::Owned(fs::read(entry.path())?),
                });
            }
        }
        files.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(files)
    }
}

/// Everything a site publishes.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Site {
    /// The files, in the order they were added.
    files: Vec<SiteFile>,
}

impl Site {
    /// An empty site.
    #[must_use]
    pub const fn new() -> Self {
        Self { files: Vec::new() }
    }

    /// Adds one file.
    pub fn add(&mut self, file: SiteFile) {
        self.files.push(file);
    }

    /// Adds several files.
    pub fn extend(&mut self, files: impl IntoIterator<Item = SiteFile>) {
        self.files.extend(files);
    }

    /// Adds the embedded Yeti build, under `vendor/yeti/`.
    #[must_use]
    pub fn with_yeti(mut self) -> Self {
        self.files.extend(YETI.iter().map(SiteFile::asset));
        self
    }

    /// The files added so far.
    #[must_use]
    pub fn files(&self) -> &[SiteFile] {
        &self.files
    }
}

/// What writing a site produced.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Summary {
    /// HTML pages written.
    pub pages: usize,
    /// Every file written, pages included.
    pub files: usize,
}

/// True when `path` is relative and stays inside the folder it is joined to.
fn is_contained(path: &str) -> bool {
    !path.is_empty()
        && Path::new(path)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

/// Writes `site` into `out`, replacing whatever was there, plus the `.nojekyll` marker that
/// stops GitHub Pages running Jekyll over branch-published output.
///
/// The folder is replaced, never merged, so a page removed from the site cannot linger.
///
/// # Errors
///
/// A file path that is absolute or climbs out of `out` (`InvalidInput`), or any I/O error.
pub fn write(site: &Site, out: &Path) -> io::Result<Summary> {
    if let Some(bad) = site.files.iter().find(|file| !is_contained(&file.path)) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "refusing to write outside the output folder: {:?}",
                bad.path
            ),
        ));
    }

    if out.exists() {
        fs::remove_dir_all(out)?;
    }
    fs::create_dir_all(out)?;

    let mut summary = Summary::default();
    for file in &site.files {
        let target = out.join(&file.path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&target, &file.bytes)?;
        summary.files += 1;
        if Path::new(&file.path)
            .extension()
            .is_some_and(|extension| extension == "html")
        {
            summary.pages += 1;
        }
    }

    fs::write(out.join(".nojekyll"), "")?;
    summary.files += 1;
    Ok(summary)
}
