use sha2::{Digest, Sha256};
use std::io::{self, Read, Write};
use std::os::unix::fs::{FileExt, MetadataExt};
use std::{
    fs,
    path::{Path, PathBuf},
};

struct IndexHeader {
    num_entries: u32,   // 3
    version: u32,       // 2
    signature: [u8; 4], // 1
}

struct IndexEntry {
    ctime: (u32, u32), // when file's metadata was last changed (seconds and nanoseconds)
    mtime: (u32, u32), // when file's content was last modified (seconds and nanoseconds)
    device: u32,
    inode: u32,
    mode: u32,
    size: u32,
    sha_hash: Vec<u8>, // SHA 256
    file_path: String, // Relative path of file from root, stored as null-terminated str
}

fn read_index() -> io::Result<(IndexHeader, Vec<IndexEntry>)> {
    let index_path = Path::new("./rit/INDEX");
    let mut buffer = vec![];

    let mut f = fs::File::open(&index_path)?;
    f.read_to_end(&mut buffer)?;

    // first 4 bytes are the signature
    let signature = &buffer[..4];
    if signature != b"DIRC" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid INDEX file format",
        ));
    }

    let version = u32::from_be_bytes(buffer[4..8].try_into().unwrap());
    let num_entries = u32::from_be_bytes(buffer[8..12].try_into().unwrap());

    let header = IndexHeader {
        num_entries,
        version,
        signature: *b"DIRC",
    };

    // Index
    let mut entries = vec![];
    let mut offset = 12;

    for _ in 0..num_entries {
        let ctime_sec = u32::from_be_bytes(buffer[offset..offset + 4].try_into().unwrap());
        let ctime_nsec = u32::from_be_bytes(buffer[offset + 4..offset + 8].try_into().unwrap());
        let mtime_sec = u32::from_be_bytes(buffer[offset + 8..offset + 12].try_into().unwrap());
        let mtime_nsec = u32::from_be_bytes(buffer[offset + 12..offset + 16].try_into().unwrap());
        let device = u32::from_be_bytes(buffer[offset + 16..offset + 20].try_into().unwrap());
        let inode = u32::from_be_bytes(buffer[offset + 20..offset + 24].try_into().unwrap());
        let mode = u32::from_be_bytes(buffer[offset + 24..offset + 28].try_into().unwrap());
        let size = u32::from_be_bytes(buffer[offset + 28..offset + 32].try_into().unwrap());
        let sha_hash = buffer[offset + 32..offset + 64].to_vec();

        let mut b: i32 = -1;
        let mut pos = offset + 64;
        while b != 0 {
            b = buffer[pos] as i32;
            pos += 1;
        }

        let file_path = String::from_utf8_lossy(&buffer[offset + 64..pos]).to_string();

        entries.push(IndexEntry {
            ctime: (ctime_sec, ctime_nsec),
            mtime: (mtime_sec, mtime_nsec),
            device,
            inode,
            mode,
            size,
            sha_hash,
            file_path,
        });

        // Align to 8-byte boundary
        offset = pos + 1;
        offset = (offset + 7) & !7;
    }

    Ok((header, entries))
}

fn get_objects_path() -> Result<PathBuf, io::Error> {
    let rit_dir = Path::new(".rit");
    let parent_dir = rit_dir.join("objects");

    if !parent_dir.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "No .rit directory found",
        ));
    }

    Ok(parent_dir)
}

fn get_hash_from_file(content: &Vec<u8>) -> (String, Vec<u8>) {
    let mut hasher = Sha256::new();
    hasher.update(content);
    let hash_result = hasher.finalize();
    (format!("{:x}", hash_result), hash_result.to_vec())
}

fn save_file_hash(file_hash: &String, parent_dir: &PathBuf, content: &Vec<u8>) -> io::Result<()> {
    let folder_name = &file_hash[..2];
    let file_name = &file_hash[2..];

    let path_name = Path::join(&parent_dir, folder_name);
    fs::create_dir(&path_name)?;

    let final_path = Path::join(&path_name, file_name);

    fs::write(final_path, &content)
}

fn add_index(header: IndexHeader, index_entries: Vec<IndexEntry>) {
    let index_path = Path::new("./rit/INDEX");
    let mut f = if index_path.exists() {
        fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(index_path)
            .expect("Failed to open INDEX")
    } else {
        fs::File::create(index_path).expect("Failed to create INDEX")
    };

    if f.metadata().unwrap().len() == 0 {
        // Write header if the file is empty
        let _ = f.write_all(&header.signature);
        let _ = f.write(&header.version.to_be_bytes());
        let _ = f.write(&header.num_entries.to_be_bytes());
    }

    // Index
    for ie in index_entries {
        let _ = f.write(&ie.ctime.0.to_be_bytes());
        let _ = f.write(&ie.ctime.1.to_be_bytes());
        let _ = f.write(&ie.mtime.0.to_be_bytes());
        let _ = f.write(&ie.mtime.1.to_be_bytes());
        let _ = f.write(&ie.device.to_be_bytes());
        let _ = f.write(&ie.inode.to_be_bytes());
        let _ = f.write(&ie.mode.to_be_bytes());
        let _ = f.write(&ie.size.to_be_bytes());
        let _ = f.write(&ie.sha_hash[..]);
        let _ = f.write(&ie.file_path.as_bytes());
    }
}

pub fn add_rit(paths: Vec<&PathBuf>) -> Result<bool, Box<dyn std::error::Error>> {
    let parent_dir;

    match get_objects_path() {
        Ok(dir) => parent_dir = dir,
        Err(e) => return Err(Box::new(e)),
    }

    let mut index_entries: Vec<IndexEntry> = vec![];
    let mut success: usize = 0;
    for path in &paths {
        if !path.exists() {
            println!("{path:?} does NOT exist; skipping...");
            continue;
        }

        if path.is_file() {
            let content = fs::read(path)?;
            let (file_hash, hash_vec) = get_hash_from_file(&content);

            match save_file_hash(&file_hash, &parent_dir, &content) {
                Ok(_) => success += 1,
                Err(_) => continue,
            }

            let md;
            match fs::metadata(path) {
                Ok(m) => md = m,
                Err(_) => continue,
            }

            let ie = IndexEntry {
                ctime: (md.ctime() as u32, md.ctime_nsec() as u32),
                mtime: (md.mtime() as u32, md.mtime_nsec() as u32),
                device: md.dev() as u32,
                inode: md.ino() as u32,
                mode: md.mode() as u32,
                size: md.size() as u32,
                sha_hash: hash_vec,

                // string must be null-terminated
                file_path: format!("{}\0", path.to_string_lossy()),
            };
            index_entries.push(ie);
        } else {
            println!("directory not supported for now");
        }
    }

    let header = IndexHeader {
        num_entries: index_entries.len() as u32,
        signature: *b"DIRC",
        version: 3,
    };

    add_index(header, index_entries);

    if success != paths.len() {
        Err("unsuccessful add operation".into())
    } else {
        Ok(true)
    }
}
