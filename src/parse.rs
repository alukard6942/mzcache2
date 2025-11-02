use std::path::{Path, PathBuf};

use crate::{
    error::MzResult,
    file::{CacheFile, parse_cachefile},
    index::{Hash, IndexFile, read_index_file},
};
use std::{
    fs::{self, File},
    io::Read,
};

pub fn parse_cache_folder(path: &PathBuf) -> MzResult<(IndexFile, Vec<(Hash, CacheFile)>)> {

    let mut index_buff = {
        let mut buf = Vec::new();
        File::open(Path::new(&path).join("index"))?.read_to_end(&mut buf)?;
        buf
    };

    let index = read_index_file(&mut index_buff)?;

    Ok((index, parse_enries(path)?))
}

pub fn parse_enries(path: &PathBuf) -> MzResult<Vec<(Hash, CacheFile)>> {
    let mut entries = Vec::new();
    for f in fs::read_dir(Path::new(&path).join("entries"))? {
        let f = f?;

        let f_path = f.path();
        let basename = f.file_name().into_string().unwrap();

        let mut file = File::open(f_path)?;

        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        let cache = match parse_cachefile(&data) {
            Ok(it) => it,
            Err(_err) => continue,
        };

        // hash conversion
        let hash: Hash = (&basename).into();
        entries.push((hash, cache));
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        io::Read,
    };

    use crate::{
        consts::DEFAULT_PATH,
        error::MzResult,
        file::parse_cachefile,
        index::{read_index_file, Hash},
    };

    // must be able to parse chace
    #[test]
    fn parsing() -> MzResult<()> {
        let path = "./cache2";

        let mut index_buff = {
            let mut buf = Vec::new();
            File::open(path.to_string() + "/index")?.read_to_end(&mut buf)?;
            buf
        };

        let index = read_index_file(&mut index_buff)?;

        // let mut doomed = Vec::new();
        let mut entries = Vec::new();

        let mut entrys_not_in_index = 0;
        for f in fs::read_dir(path.to_string() + "/entries")? {
            let f = f?;

            let f_path = f.path();
            let basename = f.file_name().into_string().unwrap();

            let mut file = File::open(f_path)?;

            let mut data = Vec::new();
            file.read_to_end(&mut data)?;

            let cache = match parse_cachefile(&data) {
                Ok(it) => it,
                Err(_err) => continue,
            };

            // hash conversion
            let hash: Hash = (&basename).into();

            let mut count = 0;
            for r in &index.records {
                if r.hash == hash {
                    count += 1
                }
            }
            assert!(count <= 1);

            if count == 0 {
                println!("hash: {hash} is missing from indexfile");
                entrys_not_in_index += 1;
            }

            entries.push((basename, cache));
        }

        let mut index_without_entry = 0;
        for r in &index.records {
            let mut cnt = 0;
            let s_hash = r.hash.to_string();
            for (hash, e) in &entries {
                if (hash.eq(&s_hash)) {
                    cnt += 1;
                }
            }

            assert!(cnt <= 1);

            if cnt == 0 {
                println!("hash: {s_hash} is missing from entries");
                index_without_entry += 1;
            }
        }

        let index_proc =
            index.records.len() as f64 / (entrys_not_in_index + index.records.len()) as f64;
        let entries_proc = entries.len() as f64 / (index_without_entry + entries.len()) as f64;

        // some hashes seam to be missing.... posibly something to do with the doom folder no idea
        assert!(index_proc >= 0.9);
        assert!(entries_proc >= 0.9);

        Ok(())
    }
}
