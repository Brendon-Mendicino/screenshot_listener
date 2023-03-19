use std::ffi::OsString;
use std::collections::HashSet;
use std::path::PathBuf;
use std::fs;


pub fn get_images(path: &PathBuf) -> HashSet<PathBuf> {
    HashSet::from_iter(
        fs::read_dir(path).unwrap()
            .into_iter()
            .filter_map(|p| {
                let p = match p {
                    Ok(val) => val.path(),
                    Err(_) => return None,
                };

                match fs::metadata(&p) {
                    Ok(ref val) => if val.is_file() { Some(p) } else { None },
                    Err(_) => None
                }
            })
    )
}


pub fn contains_img_dir(path: &PathBuf) -> bool {
    fs::read_dir(path).unwrap()
        .into_iter()
        .filter(|p| {
            let p = match p {
                Ok(val) => val,
                Err(_) => return false,
            };

            p.file_name() == OsString::from("img") && fs::metadata(p.path()).unwrap().is_dir()
        })
        .count() == 1
}

/// A directory to be included must contain a `img` subfolder
pub fn get_note_dirs(path: &PathBuf) -> HashSet<PathBuf> {
    HashSet::from_iter(
        fs::read_dir(path).unwrap()
            .into_iter()
            .filter_map(|p| {
                let p = match p {
                    Ok(val) => val.path(),
                    Err(_) => return None,
                };

                match fs::metadata(&p) {
                    Ok(ref val) => if val.is_dir() && contains_img_dir(&p) { Some(p) } else { return None },
                    Err(_) => return None
                }
            })
    )
}


pub fn get_new_images(path: &PathBuf, old_images: &HashSet<PathBuf>) -> HashSet<PathBuf> {
    let mut new_images = HashSet::<PathBuf>::new();

    fs::read_dir(path).unwrap()
        .for_each(|p| {
            match p {
                Ok(val) => {
                    if old_images.contains(&val.path()) { return }
                    new_images.insert(val.path())
                },
                Err(_) => false,
            };
        });

    new_images
}
