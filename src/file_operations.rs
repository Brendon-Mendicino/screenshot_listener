use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::path::{PathBuf, Path};
use std::str::FromStr;
pub fn contains_img_dir(path: &PathBuf) -> bool {
    fs::read_dir(path)
        .unwrap()
        .into_iter()
        .filter(|p| {
            let p = match p {
                Ok(val) => val,
                Err(_) => return false,
            };

            let img = OsString::from_str("img").unwrap();

            p.file_name() == img && fs::metadata(p.path()).unwrap().is_dir()
        })
        .count()
        == 1
}

/// A directory to be included must contain a `img` subfolder
pub fn get_note_dirs(path: &Path) -> BTreeSet<PathBuf> {
    let iter = fs::read_dir(path).unwrap().into_iter().filter_map(|p| {
        let p = match p {
            Ok(val) => val.path(),
            Err(_) => return None,
        };

        match fs::metadata(&p) {
            Ok(ref val) => {
                if val.is_dir() && contains_img_dir(&p) {
                    Some(p)
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    });

    BTreeSet::from_iter(iter)
}