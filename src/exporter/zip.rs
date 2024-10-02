use std::{
    fs::{self, File},
    io::{Read, Seek, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use walkdir::{DirEntry, WalkDir};
use zip::{result::ZipError, write::SimpleFileOptions};

pub fn zip_folder(src_dir: &Path) -> Result<PathBuf> {
    if !Path::new(src_dir).is_dir() {
        return Err(ZipError::FileNotFound.into());
    }

    let tmp_dir = Path::new("./tmp");
    fs::create_dir_all(tmp_dir)?;
    let path = get_random_filename(tmp_dir).with_extension("zip");
    let file = File::create(&path)?;

    let walkdir = WalkDir::new(src_dir);
    let it = walkdir.into_iter();

    zip_dir(
        &mut it.filter_map(|e| e.ok()),
        src_dir,
        file,
        zip::CompressionMethod::Deflated,
    )?;

    Ok(path)
}

fn get_random_filename(dir: &Path) -> PathBuf {
    loop {
        let filename = get_random_string();
        let path = dir.join(&filename);
        if !path.exists() {
            return path;
        }
    }
}

fn get_random_string() -> String {
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};

    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(30)
        .map(char::from)
        .collect()
}

fn zip_dir<T>(
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: &Path,
    writer: T,
    method: zip::CompressionMethod,
) -> Result<()>
where
    T: Write + Seek,
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = SimpleFileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let prefix = Path::new(prefix.parent().with_context(|| "No parent directory")?);
    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(prefix).unwrap();
        let path_as_string = name
            .to_str()
            .map(str::to_owned)
            .with_context(|| format!("{name:?} Is a Non UTF-8 Path"))?;
        if path.is_file() {
            zip.start_file(path_as_string, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            zip.add_directory(path_as_string, options)?;
        }
    }
    zip.finish()?;
    Ok(())
}
