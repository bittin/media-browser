
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

pub fn downscale_path(path: &std::path::PathBuf) -> std::path::PathBuf {
    let thumbpath; 
    let hashvalue: u64 = calculate_hash(path);
    let mut basename = String::from("thumbnail");
    if let Some(base) = path.file_stem() {
        basename = crate::parsers::osstr_to_string(base.to_os_string());
    }
    let filename = format!("{:016x}_{}_downscale.png", hashvalue, basename);
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

pub fn downscale_image(path: &std::path::PathBuf, max_size: u32, width: usize, height: usize) -> std::path::PathBuf {
    if let Ok(image) = image::ImageReader::open(
        std::path::PathBuf::from(path.clone())) {
        if let Ok(img) = image.decode() {
            let nwidth;
            let nheight;
            if width > height {
                nwidth = max_size;
                nheight = nwidth * height as u32 / width as u32;
            } else {
                nheight = max_size;
                nwidth = nheight * width as u32 / height as u32;
            }
            let newimg = img.resize(nwidth, nheight, image::imageops::FilterType::Lanczos3);
            let newpath = crate::thumbnails::downscale_path(&path);
            if newpath.is_file() {
                match std::fs::remove_file(newpath.clone()) {
                    Ok(()) => {},
                    Err(error) => log::error!("Failed to delete dummy file: {}", error),
                }
            }
            let file = std::fs::File::create(newpath.clone()).unwrap();
            let mut buff = std::io::BufWriter::new(file);
            let encoder = image::codecs::png::PngEncoder::new(&mut buff);
            match newimg.write_with_encoder(encoder) {
                Ok(()) => return newpath,
                Err(_error) => {},
            }
        }
    }
    path.to_owned()
}

pub fn create_thumbnail(path: &std::path::PathBuf, max_size: u32) -> String {
    let mut thumbstring = String::new();
    let thumbpath = thumbnail_path(path);
    if thumbpath.exists() {
        thumbstring = crate::parsers::osstr_to_string(thumbpath.clone().into_os_string());
        return thumbstring;
    }
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
                Err(_error) => return thumbstring,
            }
        },
        Err(_error) => return thumbstring,
    }

    thumbstring
}

pub fn create_thumbnail_downscale_if_necessary(
    path: &std::path::PathBuf, 
    tumb_size: u32, 
    max_pixel_count: u32
) -> (String, String) {
    let mut thumbstring = String::new();
    let thumbpath = thumbnail_path(path);
    let mut imagestring = crate::parsers::osstr_to_string(path.clone().into_os_string());
    if thumbpath.exists() {
        thumbstring = crate::parsers::osstr_to_string(thumbpath.clone().into_os_string());
        return (imagestring, thumbstring);
    }
    match image::ImageReader::open(path) {
        Ok(img) => {
            match img.decode() {
                Ok(image) => {
                    if image.width() > max_pixel_count || image.height() > max_pixel_count {
                        let newimage = downscale_image(
                            path, 
                            max_pixel_count, 
                            image.width() as usize, 
                            image.height() as usize);
                        imagestring = crate::parsers::osstr_to_string(newimage.clone().into_os_string());
                    }
                    let nwidth;
                    let nheight;
                    if image.width() > image.height() {
                        nwidth = tumb_size;
                        nheight = nwidth * image.height() / image.width();
                    } else {
                        nheight = tumb_size;
                        nwidth = nheight * image.width() / image.height();
                    }
                    let thumb = image::imageops::resize(&image, nwidth, nheight, image::imageops::FilterType::Lanczos3);

                    thumbstring = crate::parsers::osstr_to_string(thumbpath.clone().into_os_string());
                    let ret = thumb.save(thumbstring.clone());
                    if ret.is_err() {
                        log::error!("Failed to create thumbnail for file {}!", path.display());
                        return (String::new(), String::new());
                    }
                },
                Err(_error) => return (imagestring, thumbstring),
            }
        },
        Err(_error) => return (imagestring, thumbstring),
    }

    (imagestring, thumbstring)
}