use std::io::{self, Read, Write};
use std::path::PathBuf;
use std::{fs, path::Path};
use walkdir::{DirEntry, WalkDir};

use crate::models::errormodels::CliError;
use crate::models::indexmodels::{IndexEntry, IndexHeader};

const IGNORED_PATHS: &[&str] = &[".", ".ritignore"];

fn extract_header(b: &Vec<u8>) -> io::Result<IndexHeader> {
    // first 4 bytes are the signature
    if &b[..4] != b"DIRC" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid INDEX file format",
        ));
    }

    let version_bytes: [u8; 4] = b[4..8]
        .try_into()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let version = u32::from_be_bytes(version_bytes);

    let num_entires_byte: [u8; 4] = b[8..12]
        .try_into()
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let num_entries = u32::from_be_bytes(num_entires_byte);

    Ok(IndexHeader::new(num_entries, version, *b"DIRC"))
}

fn create_index_entry_from_bytes(buffer: &Vec<u8>, offset: usize) -> (IndexEntry, usize) {
    let ctime_sec = u32::from_be_bytes(buffer[offset..offset + 4].try_into().unwrap());
    let ctime_nsec = u32::from_be_bytes(buffer[offset + 4..offset + 8].try_into().unwrap());
    let mtime_sec = u32::from_be_bytes(buffer[offset + 8..offset + 12].try_into().unwrap());
    let mtime_nsec = u32::from_be_bytes(buffer[offset + 12..offset + 16].try_into().unwrap());
    let device = u32::from_be_bytes(buffer[offset + 16..offset + 20].try_into().unwrap());
    let inode = u32::from_be_bytes(buffer[offset + 20..offset + 24].try_into().unwrap());
    let mode = u32::from_be_bytes(buffer[offset + 24..offset + 28].try_into().unwrap());
    let size = u32::from_be_bytes(buffer[offset + 28..offset + 32].try_into().unwrap());
    let sha_hash = buffer[offset + 32..offset + 64].to_vec();

    let mut pos = offset + 64;
    let file_path_len = u32::from_be_bytes(buffer[pos..pos + 4].try_into().unwrap());
    pos += 4;

    let file_path = String::from_utf8_lossy(&buffer[pos..pos + file_path_len as usize]).to_string();
    pos += file_path_len as usize;
    (
        IndexEntry {
            ctime: (ctime_sec, ctime_nsec),
            mtime: (mtime_sec, mtime_nsec),
            device,
            inode,
            mode,
            size,
            sha_hash,
            file_path_len,
            file_path,
        },
        pos,
    )
}

pub fn read_index() -> io::Result<(IndexHeader, Vec<IndexEntry>)> {
    let index_path = Path::new("./.rit/INDEX");
    let mut f = fs::File::open(&index_path)?;

    let mut buffer = vec![];
    f.read_to_end(&mut buffer)?;

    let header = extract_header(&buffer)?;

    let mut entries = vec![];
    let mut offset = 12;

    for _ in 0..header.num_entries() {
        let (ie, pos) = create_index_entry_from_bytes(&buffer, offset);
        entries.push(ie);

        // Align to 8-byte boundary
        // "!7" is an integer mask that clears the last 3 bits
        // of a number when combined with a bitwise AND. It's
        // NOTing the 7.
        // The last 3 bits represent remainders modulo 8.
        offset = (pos + 7) & !7;
    }

    Ok((header, entries))
}

pub fn write_index(index_header: IndexHeader, index_entries: Vec<IndexEntry>) -> io::Result<bool> {
    let index_path = Path::new("./.rit/INDEX");
    let mut f = if index_path.exists() {
        fs::OpenOptions::new()
            .read(true)
            .write(true)
            // .append(true)
            .open(index_path)?
    } else {
        fs::File::create(index_path)?
    };

    // Check if file is empty, write header
    // if f.metadata().unwrap().len() == 0 {}
    f.write(&index_header.signature())?;
    f.write(&index_header.version().to_be_bytes())?;
    f.write(&index_header.num_entries().to_be_bytes())?;

    // header is always 3 * 4 bytes
    let mut pos = 12 as usize;
    // 9 * 4 + 32  bytes
    let bytes_until_file_path = 68 as usize;

    for ie in index_entries {
        f.write(&ie.ctime.0.to_be_bytes())?;
        f.write(&ie.ctime.1.to_be_bytes())?;
        f.write(&ie.mtime.0.to_be_bytes())?;
        f.write(&ie.mtime.1.to_be_bytes())?;
        f.write(&ie.device.to_be_bytes())?;
        f.write(&ie.inode.to_be_bytes())?;
        f.write(&ie.mode.to_be_bytes())?;
        f.write(&ie.size.to_be_bytes())?;
        f.write(&ie.sha_hash[..])?;
        f.write(&ie.file_path_len.to_be_bytes())?;

        pos += bytes_until_file_path;
        pos += f.write(&ie.file_path.as_bytes())?;

        let offset = pos % 8;
        if offset > 0 {
            let padding = 8 - offset;
            f.write_all(&vec![0; padding])?;
            pos += padding;
        }
    }

    Ok(true)
}

pub fn save_file_hash(
    file_hash: &String,
    objects_path: &PathBuf,
    content: &Vec<u8>,
) -> io::Result<()> {
    let folder_name = &file_hash[..3];
    let file_name = &file_hash[3..];

    let path_name = Path::join(objects_path, folder_name);
    fs::create_dir(&path_name)?;

    let final_path = Path::join(&path_name, file_name);

    fs::write(final_path, content)
}

/// Possible Errors:
/// - Path points to a directory.
/// - The file doesn’t exist.
/// - The user lacks permissions to remove the file.
pub fn delete_file_hash(objects_path: &PathBuf, file_hash: &String) -> io::Result<()> {
    let folder_name = &file_hash[..3];
    let file_name = &file_hash[3..];

    let final_path = Path::join(objects_path, folder_name).join(file_name);
    fs::remove_file(final_path)?;

    Ok(())
}

pub fn get_objects_path() -> Result<PathBuf, io::Error> {
    let rit_dir = Path::new(".rit");
    let objects_path = rit_dir.join("objects");

    if !objects_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No .rit directory found",
        ));
    }

    Ok(objects_path)
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn should_ignore(e: &DirEntry, ignore_list: &Vec<PathBuf>) -> bool {
    if let Some(name) = e.file_name().to_str() {
        ignore_list
            .iter()
            .any(|term| name.contains(term.to_str().unwrap_or("")))
    } else {
        false
    }
}

fn should_ignore_or_hidden(entry: &DirEntry, ignore_list: &Vec<PathBuf>) -> bool {
    if let Some(path_str) = entry.path().to_str() {
        if IGNORED_PATHS.contains(&path_str) {
            return false;
        }
    }
    should_ignore(entry, ignore_list) || is_hidden(entry)
}

fn get_ignore_list() -> Vec<PathBuf> {
    let mut ignore_list: Vec<PathBuf> = vec![];

    // TODO: take into account that .ritignore might not exist
    // Add entries from .ritignore file
    match std::fs::read_to_string(".ritignore") {
        Ok(res) => {
            for line in res.lines() {
                ignore_list.push(PathBuf::from(line));
            }
        }
        Err(e) => eprintln!("{}", e),
    }

    ignore_list
}

pub fn get_all_paths() -> Vec<PathBuf> {
    let ignore_list = get_ignore_list();

    let root_dir = Path::new(".");
    let mut paths = vec![];

    // search for files in "."
    if let Ok(entries) = std::fs::read_dir(root_dir) {
        for e in entries.filter_map(|e| e.ok()) {
            if e.path().is_file() {
                paths.push(e.path());
            }
        }
    }

    // search all subdirs
    for entry in WalkDir::new(root_dir)
        .into_iter()
        .filter_entry(|e| !should_ignore_or_hidden(e, &ignore_list))
        // skip the non-permitted dirs
        .filter_map(|e| e.ok())
    {
        if !entry.path().is_dir() {
            paths.push(entry.into_path());
        }
    }

    paths
}

pub fn get_config_path() -> Result<PathBuf, CliError> {
    let config_path = PathBuf::from(".rit/config");
    if !config_path.exists() {
        return Err(CliError::from(io::Error::new(
            io::ErrorKind::NotFound,
            "config file does not exist.",
        )));
    }

    Ok(config_path)
}
