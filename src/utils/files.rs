use crate::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::{Path, PathBuf};
use tokio::{
    fs::{self, File},
    io::{AsyncWriteExt, BufReader, BufWriter},
};
use walkdir::WalkDir;

// region: File parser
pub async fn load_from_toml<T>(file: impl AsRef<Path>) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let content = read_to_string(file.as_ref()).await?;

    Ok(toml::from_str(&content)?)
}
pub async fn load_from_json<T>(file: impl AsRef<Path>) -> Result<T>
where
    T: serde::de::DeserializeOwned,
{
    let content = read_to_string(file.as_ref()).await?;
    let val = serde_json::from_str(&content)?;
    Ok(val)
}
// endregion: File parser
// endregion: File parser

// region: Dir Utils

///returns true if one or more directory is created
pub async fn ensure_dir(dir: &Path) -> Result<bool> {
    if dir.is_dir() {
        Ok(false)
    } else {
        fs::create_dir_all(&dir).await?;
        Ok(true)
    }
}

pub fn list_files(
    dir: &Path,
    include_globs: Option<&[&str]>,
    exclude_globs: Option<&[&str]>,
) -> Result<Vec<PathBuf>> {
    let base_dir_exclude = base_dir_exclude_globs()?;
    // Recursive depth
    let depth = include_globs
        .map(|globs| globs.iter().any(|&g| g.contains("**")))
        .map(|v| if v { 100 } else { 1 })
        .unwrap_or(1);
    // prep globs
    let include_globs = include_globs.map(get_glob_set).transpose()?;
    let exclude_globs = exclude_globs.map(get_glob_set).transpose()?;
    // build file iterator
    let walk_dir_it = WalkDir::new(dir)
        .max_depth(depth)
        .into_iter()
        .filter_entry(|e| {
            if e.file_type().is_dir() {
                !base_dir_exclude.is_match(e.path())
            } else {
                if let Some(exclude_globs) = exclude_globs.as_ref() {
                    if exclude_globs.is_match(e.path()) {
                        return false;
                    }
                }
                match include_globs.as_ref() {
                    Some(globs) => globs.is_match(e.path()),
                    None => true,
                }
            }
        })
        .filter_map(|e| e.ok().filter(|e| e.file_type().is_file()));
    let paths = walk_dir_it.map(|e| e.into_path());
    Ok(paths.collect())
}
pub fn base_dir_exclude_globs() -> Result<GlobSet> {
    get_glob_set(&[".git", "target"])
}
pub fn get_glob_set(globs: &[&str]) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for glob in globs {
        builder.add(Glob::new(glob)?);
    }
    Ok(builder.build()?)
}
// endregion: Dir Utils

// region: Files Utils
pub async fn read_to_string(file: &Path) -> Result<String> {
    if !file.is_file() {
        return Err(format!("File not found : {}", file.display()).into());
    }
    let content = fs::read_to_string(file).await?;
    Ok(content)
}
async fn get_reader(file: &Path) -> Result<BufReader<File>> {
    let file = File::open(file).await?;
    Ok(BufReader::new(file))
}
pub async fn save_to_file<T>(file: impl AsRef<Path>, data: &T) -> Result<()>
where
    T: serde::Serialize,
{
    let file = file.as_ref();
    let mut file = File::create(file).await?;
    let data = serde_json::to_string_pretty(data)?;
    file.write(data.as_bytes()).await?;
    Ok(())
}
// endregion: Files Utils

// region: File Bundeler
pub async fn bundle_to_file(files: Vec<PathBuf>, dst_file: &Path) -> Result<()> {
    let mut writer = BufWriter::new(File::create(dst_file).await?);
    for file in files.iter() {
        if !file.is_file() {
            return Err(format!("cannot bundle '{:?}' is not a file", file).into());
        }
        let mut reader = BufReader::new(File::open(file).await?);
        let mut inner_writer = writer.get_mut();

        writer
            .write_all(format!("\n// === file path: {}\n", file.to_string_lossy()).as_bytes())
            .await?;
        //FIXME:
        tokio::io::copy(&mut reader, &mut writer).await?;
    }
    writer.flush().await?;
    Ok(())
}
// endregion: File Bundeler// region: File Bundeler
// region: File Bundeler
// region: File Bundeler
