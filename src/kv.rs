use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::{BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::{KvsError, Result};

/// The `KvStore` stores string key/value pairs.
///
/// Key/value pairs are stored in a `HashMap` in memory and not persisted to disk.
pub struct KvStore {
    buf: BufWriter<File>,

    offset: u64,
    segment: u64,

    index: HashMap<String, (u64, u64)>,
    readers: HashMap<u64, BufReader<File>>,
}

impl KvStore {
    /// Creates a `KvStore`.
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let path: PathBuf = path.into();

        // create directory if required
        fs::create_dir_all(&path)?;

        let segments = sorted_segments(&path)?;
        let mut index = HashMap::new();
        let mut readers = HashMap::new();

        for &segment in &segments {
            load_segment(&path, segment, &mut index, &mut readers)?;
        }

        let segment = segments.last().unwrap_or(&0) + 1;

        // prepare new segment log buffer
        let buf = new_segment(&path, segment)?;

        Ok(KvStore {
            buf,
            offset: 0,
            segment,
            index,
            readers,
        })
    }

    /// Applies the command to the log and in-memory index.
    fn apply(&mut self, cmd: Command) -> Result<()> {
        let res = serde_json::to_vec(&cmd)?;
        self.buf.write(&res)?;
        self.buf.flush()?;

        match cmd {
            Command::Set { key, value: _ } => {
                self.index.insert(key, (self.segment, self.offset));
            }

            Command::Remove { key } => {
                self.index.remove(&key);
            }
        };

        self.offset += res.len() as u64;

        Ok(())
    }

    /// Sets the value of a string key to a string.
    ///
    /// If the key already exists, the previous value will be overwritten.
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        self.apply(Command::Set { key, value })
    }

    /// Remove a given key.
    pub fn remove(&mut self, key: String) -> Result<()> {
        if !self.index.contains_key(&key) {
            return Err(KvsError::KeyNotFound);
        }

        self.apply(Command::Remove { key })
    }

    /// Gets the string value of a given string key.
    ///
    /// Returns `None` if the given key does not exist.
    pub fn get(&mut self, key: String) -> Result<Option<String>> {
        if !self.index.contains_key(&key) {
            return Ok(None);
        }

        let (segment, offset) = self.index.get(&key).unwrap();
        Ok(read_value(&mut self.readers, *segment, *offset)?)
    }
}

/// Constructs a path to a segment file by combining the base path with a segment number
/// Returns a `PathBuf` representing the full path to the segment file (e.g., "/base/path/123.log")
fn segment_path(path: &Path, segment: u64) -> PathBuf {
    path.join(format!("{segment}.log"))
}

/// Creates a new segment file and returns a buffered writer to it
fn new_segment(path: &Path, segment: u64) -> Result<BufWriter<File>> {
    Ok(BufWriter::new(
        OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(segment_path(path, segment))?,
    ))
}

/// Reads a value from a specific offset in a segment file
fn read_value(
    readers: &mut HashMap<u64, BufReader<File>>,
    segment: u64,
    offset: u64,
) -> Result<Option<String>> {
    let reader = match readers.get_mut(&segment) {
        None => return Ok(None),
        Some(reader) => reader,
    };

    reader.seek(SeekFrom::Start(offset))?;

    let mut stream = serde_json::Deserializer::from_reader(reader).into_iter::<Command>();

    if let Some(res) = stream.next() {
        match res? {
            Command::Set { key: _, value } => return Ok(Some(value)),
            _ => {}
        }
    }

    Ok(None)
}


/// Loads a segment file into the index map
fn load_segment(
    path: &Path,
    segment: u64,
    index: &mut HashMap<String, (u64, u64)>,
    readers: &mut HashMap<u64, BufReader<File>>,
) -> Result<()> {
    let reader = BufReader::new(File::open(segment_path(&path, segment))?);
    let mut stream = serde_json::Deserializer::from_reader(reader.get_ref()).into_iter::<Command>();

    let mut offset: u64 = 0;
    while let Some(cmd) = stream.next() {
        match cmd? {
            Command::Set { key, value: _ } => {
                index.insert(key, (segment, offset));
            }

            Command::Remove { key } => {
                index.remove(&key);
            }
        }

        offset = stream.byte_offset() as u64;
    }

    readers.insert(segment, reader);

    Ok(())
}

/// Returns a sorted list of all segment numbers in the directory
fn sorted_segments(path: &Path) -> Result<Vec<u64>> {
    let mut entries: Vec<u64> = fs::read_dir(path)?
        .flat_map(|f| -> Result<_> { Ok(f?.path()) })
        .filter(|f| f.is_file() && f.extension() == Some("log".as_ref()))
        .flat_map(|f| {
            f.file_name()
                .and_then(OsStr::to_str)
                .map(|f| f.trim_end_matches(".log"))
                .map(str::parse::<u64>)
        })
        .flatten()
        .collect();

    entries.sort();

    Ok(entries)
}

/// Represents the commands that can be stored in the log files
///
/// Each command is serialized to JSON and written to the log files.
/// - Set: Stores a key-value pair
/// - Remove: Removes a key and its associated value
#[derive(Serialize, Deserialize, Debug)]
enum Command {
    Set { key: String, value: String },
    Remove { key: String },
}
