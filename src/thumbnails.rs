
use std::hash::{DefaultHasher, Hash, Hasher};

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

pub fn thumbnail_path(path: &std::path::PathBuf) -> std::path::PathBuf {
    let thumbpath; 
    let hashvalue: u64 = calculate_hash(path);
    let mut basename = String::from("thumbnail");
    if let Some(base) = path.file_stem() {
        basename = crate::parsers::osstr_to_string(base.to_os_string());
    }
    let filename = format!("{:016x}_{}.png", hashvalue, basename);
    match dirs::data_local_dir() {
        Some(pb) => {
            let mut dir = pb.join("cosmic-media-browser").join("thumbs");
            if !dir.exists() {
                let ret = std::fs::create_dir_all(dir.clone());
                if ret.is_err() {
                    log::warn!("Failed to create directory {}", dir.display());
                    dir = dirs::home_dir().unwrap();
                }
            }
            thumbpath = dir.join(filename);
        },
        None => {
            let dir = dirs::home_dir().unwrap().join(".thumbs").join("large");
            thumbpath = dir.join(filename);
        },
    }
    thumbpath
}

pub fn create_thumbnail(path: &std::path::PathBuf, max_size: u32) -> String {
    let mut thumbstring = String::new();
    let thumbpath = thumbnail_path(path);
    match image::ImageReader::open(path) {
        Ok(img) => {
            match img.decode() {
                Ok(image) => {
                    let nwidth;
                    let nheight;
                    if image.width() > image.height() {
                        nwidth = max_size;
                        nheight = nwidth * image.height() / image.width();
                    } else {
                        nheight = max_size;
                        nwidth = nheight * image.width() / image.height();
                    }
                    let thumb = image::imageops::resize(&image, nwidth, nheight, image::imageops::FilterType::Lanczos3);

                    thumbstring = crate::parsers::osstr_to_string(thumbpath.clone().into_os_string());
                    let ret = thumb.save(thumbstring.clone());
                    if ret.is_err() {
                        log::error!("Failed to create thumbnail for file {}!", path.display());
                        return String::new();
                    }
                },
                Err(error) => return thumbstring,
            }
        },
        Err(error) => return thumbstring,
    }

    thumbstring
}