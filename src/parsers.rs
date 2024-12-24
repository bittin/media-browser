use crate::config::IconSizes;
use crate::mime_app::mime_apps;
use crate::mime_icon::{mime_for_path, mime_icon};
use crate::tab::{hidden_attribute, Item, ItemMetadata, ItemThumbnail, Location};

use chrono::NaiveDate;
use cosmic::widget::{self, Widget};
use mime_guess::mime;
use std::cell::Cell;
use std::ops::ControlFlow;
use std::path::PathBuf;
use std::time::{Duration, UNIX_EPOCH};

fn parse_nfo(nfo_file: &PathBuf, metadata: &mut crate::sql::VideoMetadata) {
    use std::fs::File;
    use std::io::BufReader;

    use xml::reader::XmlEvent;
    let file = match File::open(nfo_file) {
        Ok(ok) => ok,
        Err(err) => {
            log::warn!("failed to read entry in {:?}: {}", nfo_file.display(), err);
            return;
        }
    };

    let mut reader = xml::ParserConfig::default()
        .ignore_root_level_whitespace(false)
        .create_reader(BufReader::new(file));

    let mut prevtag = String::new();
    let mut tag = String::new();
    let mut level = 0;
    loop {
        match reader.next() {
            Ok(e) => match e {
                XmlEvent::StartDocument {
                    version, encoding, ..
                } => {
                    //println!("StartDocument({version}, {encoding})");
                }
                XmlEvent::EndDocument => {
                    //println!("EndDocument");
                    break;
                }
                XmlEvent::ProcessingInstruction { name, data } => {}
                XmlEvent::StartElement {
                    name, attributes, ..
                } => {
                    tag = name.to_string().to_ascii_lowercase();
                    match &tag as &str {
                        "actor" => {
                            prevtag = tag.clone();
                        }
                        "video" => {
                            prevtag = tag.clone();
                        }
                        "audio" => {
                            prevtag = tag.clone();
                        }
                        "subtitle" => {
                            prevtag = tag.clone();
                        }
                        _ => {}
                    }
                    level += 1;
                    /*
                    if attributes.is_empty() {
                        println!("StartElement({name})");
                    } else {
                        let attrs: Vec<_> = attributes
                            .iter()
                            .map(|a| format!("{}={:?}", &a.name, a.value))
                            .collect();
                        println!("StartElement({name} [{}])", attrs.join(", "));
                    }*/
                }
                XmlEvent::EndElement { name } => {
                    tag = name.to_string().to_ascii_lowercase();
                    if tag == prevtag {
                        prevtag.clear();
                    }
                    //println!("EndElement({name})");
                    level -= 1;
                }
                XmlEvent::Comment(_data) => {}
                XmlEvent::CData(_data) => {}
                XmlEvent::Characters(data) => {
                    let value = data.escape_debug().to_string();
                    match &tag as &str {
                        "title" => {
                            metadata.title = value.clone();
                        }
                        "plot" => {
                            metadata.description = value.clone();
                        }
                        "runtime" => {
                            let duration = match u32::from_str_radix(&value, 10) {
                                Ok(ok) => ok,
                                Err(err) => {
                                    log::warn!("failed to parse number {:?}: {}", value, err);
                                    continue;
                                }
                            };
                            metadata.duration = duration * 60;
                        }
                        "premiered" => {
                            let ret = NaiveDate::parse_from_str(&value, "%Y-%m-%d");
                            if ret.is_ok() {
                                metadata.date = ret.unwrap();
                            }
                        }
                        "director" => {
                            metadata.director.push(value.clone());
                        }
                        "name" => {
                            metadata.actors.push(value.clone());
                            prevtag.clear();
                        }
                        "width" => {
                            let val = match u32::from_str_radix(&value, 10) {
                                Ok(ok) => ok,
                                Err(err) => {
                                    log::warn!("failed to parse number {:?}: {}", value, err);
                                    continue;
                                }
                            };
                            metadata.width = val;
                        }
                        "height" => {
                            let val = match u32::from_str_radix(&value, 10) {
                                Ok(ok) => ok,
                                Err(err) => {
                                    log::warn!("failed to parse number {:?}: {}", value, err);
                                    continue;
                                }
                            };
                            metadata.height = val;
                        }
                        "durationinseconds" => {
                            let val = match u32::from_str_radix(&value, 10) {
                                Ok(ok) => ok,
                                Err(err) => {
                                    log::warn!("failed to parse number {:?}: {}", value, err);
                                    continue;
                                }
                            };
                            metadata.duration = val;
                        }
                        "language" => {
                            if &prevtag == "audio" {
                                metadata.audiolangs.push(value.clone());
                            }
                            if &prevtag == "subtitle" {
                                metadata.sublangs.push(value.clone());
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            },
            Err(e) => {
                break;
            }
        }
    }
}

fn parse_audiotags(file: &PathBuf, metadata: &mut crate::sql::AudioMetadata) {
    use audiotags::{MimeType, Tag};

    // using `default()` or `new()` alone so that the metadata format is
    // guessed (from the file extension) (in this case, Id3v2 tag is read)
    let tag = Tag::new().read_from_path(file).unwrap();

    match tag.title() {
        Some(value) => metadata.title = value.to_string(),
        None => {}
    };
    match tag.artists() {
        Some(value) => {
            for s in value {
                metadata.artist.push(s.to_string());
            }
        }
        None => {}
    };
    match tag.album_artists() {
        Some(value) => {
            for s in value {
                metadata.albumartist.push(s.to_string());
            }
        }
        None => {}
    };
    match tag.date() {
        Some(timestamp) => {
            if timestamp.month.is_some() && timestamp.day.is_some() {
                let nd = NaiveDate::from_ymd_opt(
                    timestamp.year,
                    timestamp.month.unwrap() as u32,
                    timestamp.day.unwrap() as u32,
                );
                if nd.is_some() {
                    metadata.date = nd.unwrap();
                }
            } else {
                let nd = NaiveDate::from_ymd_opt(timestamp.year, 1, 1);
                if nd.is_some() {
                    metadata.date = nd.unwrap();
                }
            }
        }
        None => {}
    };
    match tag.duration() {
        Some(value) => metadata.duration = value as u32,
        None => {}
    };
    match tag.album_cover() {
        Some(picture) => {
            let mut thumbpath = metadata.path.clone();
            thumbpath.push_str(".png");
            if !PathBuf::from(&thumbpath).is_file() {
                match picture.mime_type {
                    MimeType::Jpeg => {
                        match image::load_from_memory_with_format(
                            picture.data,
                            image::ImageFormat::Jpeg,
                        ) {
                            Ok(buf) => {
                                let _ = buf.save_with_format(&thumbpath, image::ImageFormat::Png);
                            }
                            Err(error) => {
                                log::warn!("failed to read audio album art jpeg: {}", error);
                            }
                        }
                    }
                    MimeType::Png => {
                        match image::load_from_memory_with_format(
                            picture.data,
                            image::ImageFormat::Png,
                        ) {
                            Ok(buf) => {
                                let _ = buf.save_with_format(&thumbpath, image::ImageFormat::Png);
                            }
                            Err(error) => {
                                log::warn!("failed to read audio album art jpeg: {}", error);
                            }
                        }
                    }
                    MimeType::Bmp => {
                        match image::load_from_memory_with_format(
                            picture.data,
                            image::ImageFormat::Jpeg,
                        ) {
                            Ok(buf) => {
                                let _ = buf.save_with_format(&thumbpath, image::ImageFormat::Bmp);
                            }
                            Err(error) => {
                                log::warn!("failed to read audio album art jpeg: {}", error);
                            }
                        }
                    }
                    MimeType::Gif => {
                        match image::load_from_memory_with_format(
                            picture.data,
                            image::ImageFormat::Gif,
                        ) {
                            Ok(buf) => {
                                let _ = buf.save_with_format(&thumbpath, image::ImageFormat::Png);
                            }
                            Err(error) => {
                                log::warn!("failed to read audio album art jpeg: {}", error);
                            }
                        }
                    }
                    MimeType::Tiff => {
                        match image::load_from_memory_with_format(
                            picture.data,
                            image::ImageFormat::Tiff,
                        ) {
                            Ok(buf) => {
                                let _ = buf.save_with_format(&thumbpath, image::ImageFormat::Png);
                            }
                            Err(error) => {
                                log::warn!("failed to read audio album art jpeg: {}", error);
                            }
                        }
                    }
                }
            }
            metadata.poster = thumbpath;
        }
        None => {}
    };
}

pub fn item_from_entry(
    path: PathBuf,
    name: String,
    metadata: std::fs::Metadata,
    sizes: IconSizes,
) -> Item {
    let mut display_name = Item::display_name(&name);

    let hidden = name.starts_with(".") || hidden_attribute(&metadata);

    let (mime, icon_handle_grid, icon_handle_list, icon_handle_list_condensed) =
        if metadata.is_dir() {
            (
                //TODO: make this a static
                "inode/directory".parse().unwrap(),
                crate::tab::folder_icon(&path, sizes.grid()),
                crate::tab::folder_icon(&path, sizes.list()),
                crate::tab::folder_icon(&path, sizes.list_condensed()),
            )
        } else {
            let mime = mime_for_path(&path);
            //TODO: clean this up, implement for trash
            let icon_name_opt = if mime == "application/x-desktop" {
                let (desktop_name_opt, icon_name_opt) = crate::tab::parse_desktop_file(&path);
                if let Some(desktop_name) = desktop_name_opt {
                    display_name = Item::display_name(&desktop_name);
                }
                icon_name_opt
            } else {
                None
            };
            if let Some(icon_name) = icon_name_opt {
                (
                    mime.clone(),
                    widget::icon::from_name(&*icon_name)
                        .size(sizes.grid())
                        .handle(),
                    widget::icon::from_name(&*icon_name)
                        .size(sizes.list())
                        .handle(),
                    widget::icon::from_name(&*icon_name)
                        .size(sizes.list_condensed())
                        .handle(),
                )
            } else {
                (
                    mime.clone(),
                    mime_icon(mime.clone(), sizes.grid()),
                    mime_icon(mime.clone(), sizes.list()),
                    mime_icon(mime, sizes.list_condensed()),
                )
            }
        };

    let open_with = mime_apps(&mime);

    let thumbnail_opt = if mime.type_() == mime::IMAGE {
        if mime.subtype() == mime::SVG {
            Some(ItemThumbnail::Svg)
        } else {
            None
        }
    } else {
        Some(ItemThumbnail::NotImage)
    };

    let children = if metadata.is_dir() {
        //TODO: calculate children in the background (and make it cancellable?)
        match std::fs::read_dir(&path) {
            Ok(entries) => entries.count(),
            Err(err) => {
                log::warn!("failed to read directory {:?}: {}", path, err);
                0
            }
        }
    } else {
        0
    };

    Item {
        name,
        display_name,
        metadata: ItemMetadata::Path { metadata, children },
        hidden,
        location_opt: Some(Location::Path(path)),
        mime,
        icon_handle_grid,
        icon_handle_list,
        icon_handle_list_condensed,
        open_with,
        thumbnail_opt,
        button_id: widget::Id::unique(),
        pos_opt: Cell::new(None),
        rect_opt: Cell::new(None),
        selected: false,
        overlaps_drag_rect: false,
    }
}

pub fn item_from_path<P: Into<PathBuf>>(path: P, sizes: IconSizes) -> Result<Item, String> {
    let path = path.into();
    let name_os = path
        .file_name()
        .ok_or_else(|| format!("failed to get file name from path {:?}", path))?;
    let name = name_os
        .to_str()
        .ok_or_else(|| {
            format!(
                "failed to parse file name for {:?}: {:?} is not valid UTF-8",
                path, name_os
            )
        })?
        .to_string();
    let metadata = std::fs::metadata(&path)
        .map_err(|err| format!("failed to read metadata for {:?}: {}", path, err))?;
    Ok(item_from_entry(path, name, metadata, sizes))
}

pub fn slice_u8_to_vec_string(utfvec: Vec<u8>) -> Vec<String> {
    let mut v = Vec::new();
    let mut null_positions = Vec::new();
    for i in 0..utfvec.len() {
        if utfvec[i] == 0 {
            null_positions.push(i);
        }
    }
    if null_positions.len() < 1 {
        match String::from_utf8(utfvec) {
            Ok(string) => v.push(string),
            Err(error) => log::error!("failed to convert string: {}", error),
        }
        return v
    }
    let mut start = 0;
    for end in null_positions {
        let mut partial: Vec<u8> = Vec::new();
        partial.extend_from_slice(&utfvec[start..end]);
        match String::from_utf8(partial) {
            Ok(string) => v.push(string),
            Err(error) => log::error!("failed to convert string: {}", error),
        }
        start = end;
    }
    v
}

fn video_metadata(filepath: &str) -> crate::sql::VideoMetadata {
    let mut meta = crate::sql::VideoMetadata {
        ..Default::default()
    };
    meta.path = filepath.to_string();
    let basename;
    let fp = PathBuf::from(filepath);
    if let Some(bn) = fp.file_stem() {
        basename = osstr_to_string(bn.to_os_string());
    } else if let Some(bn) = fp.file_name() {
        basename = osstr_to_string(bn.to_os_string());
    } else {
        basename = filepath.to_string();
    }
    meta.name = basename.clone();
    meta.title = basename.clone();
    let mut cmd_runner = crate::cmd::CmdRunner::new(&format!("ffmpeg -i \"{}\"", filepath));
    if let Ok((stdout, _stderr)) = cmd_runner.run_with_output() {
        // let ffmpeg_output = std::process::Command::new("ffmpeg")
        //    .args(["-i", filepath])
        //    .output()
        //    .expect("failed to execute process");
        //let lines = slice_u8_to_vec_string(ffmpeg_output.stderr);
        for line in stdout { 
            if let Ok(re_duration) = regex::Regex::new(
                r"(?i)\s*Duration:\s+(?P<hours>\d+):(?P<minutes>\d+):(?P<seconds>\d+).\d+\s*,",
            ) {
                if re_duration.is_match(&line) {
                    let caps = re_duration.captures(&line).unwrap();
                    let hours = string_to_uint(&caps["hours"]);
                    let minutes = string_to_uint(&caps["minutes"]);
                    let seconds = string_to_uint(&caps["seconds"]);
                    meta.duration = hours * 3600 + minutes * 60 + seconds;
                }
            }
            if let Ok(re_video) =
            regex::Regex::new(r"(?i), (?P<width>\d+)x(?P<height>\d+)")
            {
                if re_video.is_match(&line) {
                    let caps = re_video.captures(&line).unwrap();
                    meta.width = string_to_uint(&caps["width"]);
                    meta.height = string_to_uint(&caps["height"]);
                }
            }
            if let Ok(re_audio) = regex::Regex::new(r"(?i), \((?P<language>\w+)\):\s*Audio") {
                if re_audio.is_match(&line) {
                    let caps = re_audio.captures(&line).unwrap();
                    meta.audiolangs.push(caps["language"].to_string());
                }
            }
            if let Ok(re_sub) = regex::Regex::new(r"(?i), \((?P<language>\w+)\):\s*Subtitle") {
                if re_sub.is_match(&line) {
                    let caps = re_sub.captures(&line).unwrap();
                    meta.sublangs.push(caps["language"].to_string());
                }
            }
        }
    }

    meta
}

pub fn string_to_uint(mystring: &str) -> u32 {
    let u = 0;
    if mystring.trim().len() == 0 {
        return u;
    }
    match u32::from_str_radix(mystring, 10) {
        Ok(ret) => return ret,
        Err(_) => {
            log::warn!("Parsing of {} into Integer failed\n", mystring)
        }
    }
    u
}

pub fn string_to_float(mystring: &str) -> f32 {
    let f = 0.0;
    if mystring.trim().len() == 0 {
        return f;
    }
    match mystring.parse::<f32>() {
        Ok(ret) => return ret,
        Err(_) => {
            log::warn!("Parsing of {} into flaot failed\n", mystring)
        }
    }
    f
}

fn timecode_to_ffmpeg_time(timecode: u32) -> String {
    let hours = timecode / 3600;
    let minutes = (timecode - hours * 3600) / 60;
    let seconds = timecode - hours * 3600 - minutes * 60;
    format!("{:02}:{:02}:{:02}.000", hours, minutes, seconds)
}

fn create_screenshots(filepath: &str) -> Option<crate::sql::VideoMetadata> {
    let mut meta = video_metadata(filepath);
    let timecode = meta.duration / 10;
    let outputpattern = format!("{}_%03d.jpeg", filepath);
    let output = format!("{}_001.jpeg", filepath);
    let outputpath = PathBuf::from(&output);
    let time = timecode_to_ffmpeg_time(timecode);
    if outputpath.is_file() {
        //let ret = std::fs::remove_file(&output);
        //if ret.is_err() {
        //    log::error!("could not delete file {}", output);
        //}
        meta.poster = output;
        return Some(meta);
    }

    match std::process::Command::new("ffmpeg")
        .args([
            "-ss",
            &time,
            "-i",
            filepath,
            "-frames:v",
            "1",
            "-q:v",
            "2",
            &outputpattern,
        ])
        .output() {
            Ok(out) => {
                match String::from_utf8(out.stderr) {
                    Ok(text) => log::warn!("{}", &text),
                    Err(_error) => {},
                }
            }, 
            Err(error) => log::error!("Failed to generate Screenshot {}: {}", output, error),
    }
    if !outputpath.is_file() {
        log::error!(
            "Failed to create screenshot: {}",
            format!("ffmpeg -ss {} -i {} -frames:v 1 -q:v 2 ", time, filepath)
        );
        return None;
    }
    meta.poster = output;
    Some(meta)
}

pub fn item_from_video(
    path: PathBuf,
    name: String,
    metadata: &std::fs::Metadata,
    sizes: IconSizes,
    known_files: &mut std::collections::BTreeMap<PathBuf, crate::sql::FileMetadata>,
    connection: &mut rusqlite::Connection,
) -> Item {
    let mut videometadata = crate::sql::VideoMetadata {..Default::default()};
    videometadata.name = name.clone();
    let mut refresh = false;
    let filepath = osstr_to_string(path.clone().into_os_string());
    if known_files.contains_key(&path) {
        if let Ok(modified) = metadata.modified() {
            if let Ok(new_date) = modified.duration_since(UNIX_EPOCH) {
                let filedata = &known_files[&path];
                let new_seconds_since_epoch = new_date.as_secs();
                if new_seconds_since_epoch > filedata.modification_time {
                    refresh = true;
                }
            }
            if refresh {
                // file is newer
                if let Some(vmd) = create_screenshots(&osstr_to_string(path.clone().into_os_string())) {
                    videometadata = vmd;
                    crate::sql::insert_video(connection, &mut videometadata, metadata, known_files);
                }
                crate::sql::update_video(connection, &mut videometadata, metadata, known_files);
            } else {
                videometadata = crate::sql::video(connection, &filepath, known_files);
            }
        }
    } else {
        if let Some(vmd) = create_screenshots(&osstr_to_string(path.clone().into_os_string())) {
            videometadata = vmd;
            crate::sql::insert_video(connection, &mut videometadata, metadata, known_files);
        }
    }
    let mut display_name = Item::display_name(&name);

    let hidden = name.starts_with(".") || hidden_attribute(&metadata);

    let (mime, icon_handle_grid, icon_handle_list, icon_handle_list_condensed) = if metadata
        .is_dir()
    {
        (
            //TODO: make this a static
            "inode/directory".parse().unwrap(),
            crate::tab::folder_icon(&path, sizes.grid()),
            crate::tab::folder_icon(&path, sizes.list()),
            crate::tab::folder_icon(&path, sizes.list_condensed()),
        )
    } else {
        if videometadata.path.len() > 0 {
            let thumbpath = PathBuf::from(&videometadata.poster);
            let thumbmime = mime_for_path(thumbpath.clone());
            let filemime = mime_for_path(filepath.clone());
            if videometadata.poster.len() > 0 {
                let thumbpath = PathBuf::from(&videometadata.poster);
                (
                    filemime.clone(),
                    widget::icon::from_path(thumbpath.clone()),
                    widget::icon::from_path(thumbpath.clone()),
                    widget::icon::from_path(thumbpath.clone()),
                )
            } else {
                (
                    filemime.clone(),
                    mime_icon(filemime.clone(), sizes.grid()),
                    mime_icon(filemime.clone(), sizes.list()),
                    mime_icon(filemime.clone(), sizes.list_condensed()),
                )
            }
        } else {
            let mime = mime_for_path(&path);
            //TODO: clean this up, implement for trash
            let icon_name_opt = if mime == "application/x-desktop" {
                let (desktop_name_opt, icon_name_opt) = crate::tab::parse_desktop_file(&path);
                if let Some(desktop_name) = desktop_name_opt {
                    display_name = Item::display_name(&desktop_name);
                }
                icon_name_opt
            } else {
                None
            };
            if let Some(icon_name) = icon_name_opt {
                (
                    mime.clone(),
                    widget::icon::from_name(&*icon_name)
                        .size(sizes.grid())
                        .handle(),
                    widget::icon::from_name(&*icon_name)
                        .size(sizes.list())
                        .handle(),
                    widget::icon::from_name(&*icon_name)
                        .size(sizes.list_condensed())
                        .handle(),
                )
            } else {
                (
                    mime.clone(),
                    mime_icon(mime.clone(), sizes.grid()),
                    mime_icon(mime.clone(), sizes.list()),
                    mime_icon(mime, sizes.list_condensed()),
                )
            }
        }
    };

    let open_with = mime_apps(&mime);

    let thumbnail_opt = if mime.type_() == mime::IMAGE {
        if mime.subtype() == mime::SVG {
            Some(ItemThumbnail::Svg)
        } else {
            None
        }
    } else {
        Some(ItemThumbnail::NotImage)
    };

    let children = if metadata.is_dir() {
        //TODO: calculate children in the background (and make it cancellable?)
        match std::fs::read_dir(&path) {
            Ok(entries) => entries.count(),
            Err(err) => {
                log::warn!("failed to read directory {:?}: {}", path, err);
                0
            }
        }
    } else {
        0
    };

    Item {
        name,
        display_name,
        metadata: ItemMetadata::Path { metadata: metadata.clone(), children },
        hidden,
        location_opt: Some(Location::Path(path)),
        mime,
        icon_handle_grid,
        icon_handle_list,
        icon_handle_list_condensed,
        open_with,
        thumbnail_opt,
        button_id: widget::Id::unique(),
        pos_opt: Cell::new(None),
        rect_opt: Cell::new(None),
        selected: false,
        overlaps_drag_rect: false,
    }
}

pub fn item_from_nfo(
    nfo_file: PathBuf,
    metadata: &mut crate::sql::VideoMetadata,
    statdata: &std::fs::Metadata,
    sizes: IconSizes,
    known_files: &mut std::collections::BTreeMap<PathBuf, crate::sql::FileMetadata>,
    connection: &mut rusqlite::Connection,
) -> Item {
    let filepath = PathBuf::from(&metadata.path);
    let basename;
    if let Some(bn) = filepath.file_stem() {
        basename = osstr_to_string(bn.to_os_string());
    } else {
        if let Some(bn) = filepath.file_name() {
            basename = osstr_to_string(bn.to_os_string());
        } else {
            basename = osstr_to_string(filepath.clone().into_os_string());
        }
    }
    if known_files.contains_key(&filepath) {
        let mut refresh = false;
        if let Ok(modified) = statdata.modified() {
            if let Ok(new_date) = modified.duration_since(UNIX_EPOCH) {
                let filedata = &known_files[&filepath];
                let new_seconds_since_epoch = new_date.as_secs();
                if new_seconds_since_epoch > filedata.modification_time {
                    refresh = true;
                }
            }
            if refresh {
                // file is newer
                parse_nfo(&nfo_file, metadata);
                metadata.name = basename.clone();
                crate::sql::update_video(connection, metadata, statdata, known_files);
            } else {
                *metadata = crate::sql::video(connection, &metadata.path, known_files);
            }
        }
    } else {
        parse_nfo(&nfo_file, metadata);
        metadata.name = basename.clone();
        crate::sql::insert_video(connection, metadata, statdata, known_files);
    }
    
    let name;
    if metadata.title.len() == 0 {
        name = basename.clone();
    } else {
        name = metadata.title.clone();
    }

    let display_name = Item::display_name(&name);

    let hidden = name.starts_with(".") || hidden_attribute(&statdata);

     let (mime, icon_handle_grid, icon_handle_list, icon_handle_list_condensed) = {
        let thumbpath = PathBuf::from(&metadata.poster);
        let thumbmime = mime_for_path(thumbpath.clone());
        let filemime = mime_for_path(filepath.clone());
        if metadata.poster.len() > 0 {
            (
                filemime.clone(),
                widget::icon::from_path(thumbpath.clone()),
                widget::icon::from_path(thumbpath.clone()),
                widget::icon::from_path(thumbpath.clone()),
            )
        } else {
            (
                filemime.clone(),
                mime_icon(thumbmime.clone(), sizes.grid()),
                mime_icon(thumbmime.clone(), sizes.list()),
                mime_icon(thumbmime.clone(), sizes.list_condensed()),
            )
        }
    };

    let open_with = mime_apps(&mime);

    let thumbnail_opt = if mime.type_() == mime::IMAGE {
        if mime.subtype() == mime::SVG {
            Some(ItemThumbnail::Svg)
        } else {
            None
        }
    } else {
        Some(ItemThumbnail::NotImage)
    };

    let children = if statdata.is_dir() {
        //TODO: calculate children in the background (and make it cancellable?)
        match std::fs::read_dir(PathBuf::from(&metadata.path)) {
            Ok(entries) => entries.count(),
            Err(err) => {
                log::warn!("failed to read directory {:?}: {}", metadata.path, err);
                0
            }
        }
    } else {
        0
    };

    Item {
        name,
        display_name,
        metadata: ItemMetadata::Path {
            metadata: statdata.clone(),
            children,
        },
        hidden,
        location_opt: Some(Location::Path(PathBuf::from(&metadata.path))),
        mime,
        icon_handle_grid,
        icon_handle_list,
        icon_handle_list_condensed,
        open_with,
        thumbnail_opt,
        button_id: widget::Id::unique(),
        pos_opt: Cell::new(None),
        rect_opt: Cell::new(None),
        selected: false,
        overlaps_drag_rect: false,
    }
}

pub fn item_from_audiotags(
    nfo_file: PathBuf,
    metadata: &mut crate::sql::AudioMetadata,
    statdata: &std::fs::Metadata,
    sizes: IconSizes,
    known_files: &mut std::collections::BTreeMap<PathBuf, crate::sql::FileMetadata>,
    connection: &mut rusqlite::Connection,
) -> Item {
    let filepath = PathBuf::from(&metadata.path);
    let basename;
    if let Some(bn) = filepath.file_stem() {
        basename = osstr_to_string(bn.to_os_string());
    } else {
        if let Some(bn) = filepath.file_name() {
            basename = osstr_to_string(bn.to_os_string());
        } else {
            basename = osstr_to_string(filepath.clone().into_os_string());
        }
    }
    if known_files.contains_key(&filepath) {
        let mut refresh = false;
        if let Ok(modified) = statdata.modified() {
            if let Ok(new_date) = modified.duration_since(UNIX_EPOCH) {
                let filedata = &known_files[&filepath];
                let new_seconds_since_epoch = new_date.as_secs();
                if new_seconds_since_epoch > filedata.modification_time {
                    refresh = true;
                }
            }
            if refresh {
                // file is newer
                parse_audiotags(&nfo_file, metadata);
                metadata.name = basename.clone();
                crate::sql::update_audio(connection, metadata, statdata, known_files);
            } else {
                *metadata = crate::sql::audio(connection, &metadata.path, known_files);
            }
        }
    } else {
        parse_audiotags(&nfo_file, metadata);
        metadata.name = basename.clone();
        crate::sql::insert_audio(connection, metadata, statdata, known_files);
    }

    let name;
    if metadata.title.len() == 0 {
        name = basename.clone();
    } else {
        name = metadata.title.clone();
    }

    let mut display_name = Item::display_name(&name);

    let hidden = name.starts_with(".") || hidden_attribute(&statdata);

    let thumbpath;
    if metadata.poster.len() == 0 {
        // generate thumbnail
        thumbpath = PathBuf::from(&metadata.path);
    } else {
        thumbpath = PathBuf::from(&metadata.poster);
    }

    let thumbmime = mime_for_path(thumbpath.clone());
    let audiomime = mime_for_path(filepath.clone());

    let (mime, icon_handle_grid, icon_handle_list, icon_handle_list_condensed) = {
        //TODO: clean this up, implement for trash
        let icon_name_opt = if audiomime == "application/x-desktop" {
            let (desktop_name_opt, icon_name_opt) = crate::tab::parse_desktop_file(&filepath);
            if let Some(desktop_name) = desktop_name_opt {
                display_name = Item::display_name(&desktop_name);
            }
            icon_name_opt
        } else {
            None
        };

        if metadata.poster.len() > 0 {
            let thumbpath = PathBuf::from(&metadata.poster);
            (
                audiomime.clone(),
                widget::icon::from_path(thumbpath.clone()),
                widget::icon::from_path(thumbpath.clone()),
                widget::icon::from_path(thumbpath.clone()),
            )
        } else {
            (
                audiomime.clone(),
                mime_icon(thumbmime.clone(), sizes.grid()),
                mime_icon(thumbmime.clone(), sizes.list()),
                mime_icon(thumbmime.clone(), sizes.list_condensed()),
            )
        }
    };

    let open_with = mime_apps(&audiomime);

    let thumbnail_opt = if thumbmime.type_() == mime::IMAGE {
        if thumbmime.subtype() == mime::SVG {
            Some(ItemThumbnail::Svg)
        } else {
            None
        }
    } else {
        Some(ItemThumbnail::NotImage)
    };

    let children = if statdata.is_dir() {
        //TODO: calculate children in the background (and make it cancellable?)
        match std::fs::read_dir(PathBuf::from(&metadata.path)) {
            Ok(entries) => entries.count(),
            Err(err) => {
                log::warn!("failed to read directory {:?}: {}", metadata.path, err);
                0
            }
        }
    } else {
        0
    };

    Item {
        name,
        display_name,
        metadata: ItemMetadata::Path {
            metadata: statdata.clone(),
            children,
        },
        hidden,
        location_opt: Some(Location::Path(PathBuf::from(&metadata.path))),
        mime,
        icon_handle_grid,
        icon_handle_list,
        icon_handle_list_condensed,
        open_with,
        thumbnail_opt,
        button_id: widget::Id::unique(),
        pos_opt: Cell::new(None),
        rect_opt: Cell::new(None),
        selected: false,
        overlaps_drag_rect: false,
    }
}

pub fn osstr_to_string(osstr: std::ffi::OsString) -> String {
    match osstr.to_str() {
        Some(str) => return str.to_string(),
        None => {}
    }
    String::new()
}

pub fn scan_files(
    special_files: &mut std::collections::BTreeSet<PathBuf>,
    path: PathBuf,
    items: &mut Vec<Item>,
    sizes: IconSizes,
) -> ControlFlow<()> {
    if special_files.contains(&path) {
        return ControlFlow::Break(());
    }
    let name;
    if let Some(bn) = path.file_name() {
        name = crate::parsers::osstr_to_string(bn.to_os_string());
    } else {
        name = crate::parsers::osstr_to_string(path.clone().into_os_string());
    }
    let metadata = match std::fs::metadata(&path) {
        Ok(ok) => ok,
        Err(err) => {
            log::warn!("failed to read metadata for entry at {:?}: {}", path, err);
            return ControlFlow::Break(());
        }
    };
    special_files.insert(path.clone());
    items.push(crate::parsers::item_from_entry(path, name, metadata, sizes));

    ControlFlow::Continue(())
}

pub fn scan_directories(
    special_files: &mut std::collections::BTreeSet<PathBuf>,
    path: PathBuf,
    items: &mut Vec<Item>,
    sizes: IconSizes,
) -> ControlFlow<()> {
    if special_files.contains(&path.clone()) {
        return ControlFlow::Break(());
    }
    let name;
    if let Some(bn) = path.file_name() {
        name = crate::parsers::osstr_to_string(bn.to_os_string());
    } else {
        name = crate::parsers::osstr_to_string(path.clone().into_os_string());
    }
    let metadata = match std::fs::metadata(&path) {
        Ok(ok) => ok,
        Err(err) => {
            log::warn!("failed to read metadata for entry at {:?}: {}", path, err);
            return ControlFlow::Break(());
        }
    };
    special_files.insert(path.clone());
    items.push(crate::parsers::item_from_entry(path, name, metadata, sizes));
    ControlFlow::Continue(())
}

pub fn scan_videos(
    special_files: &mut std::collections::BTreeSet<PathBuf>,
    path: PathBuf,
    items: &mut Vec<Item>,
    sizes: IconSizes,
    known_files: &mut std::collections::BTreeMap<PathBuf, crate::sql::FileMetadata>,
    connection: &mut rusqlite::Connection,
) -> ControlFlow<()> {
    if special_files.contains(&path.clone()) {
        return ControlFlow::Break(());
    }
    let name;
    if let Some(bn) = path.file_name() {
        name = crate::parsers::osstr_to_string(bn.to_os_string());
    } else {
        name = crate::parsers::osstr_to_string(path.clone().into_os_string());
    }
    let metadata = match std::fs::metadata(&path) {
        Ok(ok) => ok,
        Err(err) => {
            log::warn!("failed to read metadata for entry at {:?}: {}", path, err);
            return ControlFlow::Break(());
        }
    };
    special_files.insert(path.clone());
    let mut thumb = osstr_to_string(path.clone().into_os_string());
    thumb.push_str("_001.jpeg");
    special_files.insert(PathBuf::from(&thumb));
    items.push(crate::parsers::item_from_video(path, name, &metadata, sizes, known_files, connection));

    ControlFlow::Continue(())
}

pub fn scan_audiotags(
    audio: PathBuf,
    special_files: &mut std::collections::BTreeSet<PathBuf>,
    items: &mut Vec<Item>,
    sizes: IconSizes,
    known_files: &mut std::collections::BTreeMap<PathBuf, crate::sql::FileMetadata>,
    connection: &mut rusqlite::Connection,
) -> ControlFlow<()> {
    if special_files.contains(&audio.clone()) {
        return ControlFlow::Break(());
    }
    let mut meta_data = crate::sql::AudioMetadata {
        ..Default::default()
    };
    meta_data.path = osstr_to_string(audio.clone().into_os_string());
    if meta_data.path.len() == 0 {
        return ControlFlow::Break(());
    }
    let statdata = match std::fs::metadata(&meta_data.path) {
        Ok(ok) => ok,
        Err(err) => {
            log::warn!(
                "failed to read metadata for entry at {:?}: {}",
                meta_data.path,
                err
            );
            return ControlFlow::Break(());
        }
    };
    special_files.insert(audio.clone());
    let mut thumbstr = osstr_to_string(audio.clone().into_os_string());
    thumbstr.push_str(".png");
    let thumb = PathBuf::from(&thumbstr);
    if thumb.exists() {
        special_files.insert(thumb.clone());
        let poster = crate::thumbnails::create_thumbnail(&thumb, 256);
        if poster.len() > 0 {
            meta_data.poster = poster.clone();
        }
    }

    items.push(crate::parsers::item_from_audiotags(
        audio,
        &mut meta_data,
        &statdata,
        sizes,
        known_files,
        connection,
    ));

    ControlFlow::Continue(())
}

pub fn scan_single_nfo_dir(
    dp: &PathBuf,
    tab_path: &PathBuf,
    special_files: &mut std::collections::BTreeSet<PathBuf>,
    justdirs: &mut Vec<PathBuf>,
    items: &mut Vec<Item>,
    sizes: IconSizes,
    known_files: &mut std::collections::BTreeMap<PathBuf, crate::sql::FileMetadata>,
    connection: &mut rusqlite::Connection,
) -> ControlFlow<()> {
    if special_files.contains(&dp.clone()) {
        return ControlFlow::Break(());
    }
    let mut meta_data = crate::sql::VideoMetadata {
        ..Default::default()
    };
    let mut movie = 0;
    let mut poster = 0;
    let mut nfo = 0;
    let mut nfo_file = dp.clone().join("movie.nfo");
    let mut contents = Vec::new();
    match std::fs::read_dir(dp) {
        Ok(entries) => {
            for entry_res in entries {
                let entry = match entry_res {
                    Ok(ok) => ok,
                    Err(err) => {
                        log::warn!("failed to read entry in {:?}: {}", tab_path, err);
                        continue;
                    }
                };
                let path = entry.path();
                contents.push(path);
            }
            if contents.len() > 13 {
                justdirs.push(dp.clone());
                return ControlFlow::Break(())
            }

            for path in contents.iter() {
                let f = osstr_to_string(path.clone().into_os_string()).to_ascii_lowercase();
                if f.contains("poster.") {
                    if poster > 0 {
                        justdirs.push(dp.clone());
                        return ControlFlow::Break(());
                    }
                    meta_data.poster = osstr_to_string(path.clone().into_os_string());
                    poster += 1;
                } else if f.contains(".srt") {
                    meta_data.subtitles.push(f.clone());
                } else if f.contains(".nfo") {
                    if nfo > 0 {
                        justdirs.push(dp.clone());
                        return ControlFlow::Break(());
                    }
                    nfo_file = path.clone();
                    nfo += 1;
                } else if f.ends_with(".mkv") || f.ends_with(".mp4") || f.ends_with(".webm") {
                    if movie > 0 {
                        justdirs.push(dp.clone());
                        return ControlFlow::Break(());
                    }
                    meta_data.path = osstr_to_string(path.clone().into_os_string());
                    if meta_data.name.len() == 0 {
                        if let Some(file) = path.clone().file_stem() {
                            meta_data.name = osstr_to_string(file.to_os_string());
                        } else {
                            meta_data.name = osstr_to_string(path.clone().into_os_string());
                        }
                    }
                    movie += 1;
                }
            }
        }
        Err(err) => {
            log::warn!("failed to read directory {:?}: {}", tab_path, err);
        }
    }
    if !PathBuf::from(&nfo_file).exists() {
        justdirs.push(dp.clone());
        return ControlFlow::Break(())
    }
    if meta_data.path.len() == 0 {
        justdirs.push(dp.clone());
        return ControlFlow::Break(());
    }
    if meta_data.poster.len() == 0 {
        if let Some(meta) = create_screenshots(&meta_data.path) {
            meta_data.poster = meta.poster.clone();
        } else {
            log::error!("Failed to find poster or create a screenshot for movie {}", &meta_data.path);
            return ControlFlow::Break(());
        }
    }
    special_files.insert(dp.clone());
    for path in contents.iter() {
        special_files.insert(path.clone());
    }
    let thumb = PathBuf::from(&meta_data.poster);
    if thumb.exists() {
        let poster = crate::thumbnails::create_thumbnail(&thumb, 256);
        if poster.len() > 0 {
            meta_data.poster = poster.clone();
        }
    }

    let statdata = match std::fs::metadata(&meta_data.path) {
        Ok(ok) => ok,
        Err(err) => {
            log::warn!(
                "failed to read metadata for entry at {:?}: {}",
                meta_data.path,
                err
            );
            return ControlFlow::Break(());
        }
    };
    items.push(crate::parsers::item_from_nfo(
        nfo_file,
        &mut meta_data,
        &statdata,
        sizes,
        known_files,
        connection,
    ));
    // test if we have a single movie with NFO in this dir

    ControlFlow::Continue(())
}

pub fn scan_nfos_in_dir(
    video: String,
    all: &Vec<PathBuf>,
    special_files: &mut std::collections::BTreeSet<PathBuf>,
    items: &mut Vec<Item>,
    sizes: IconSizes,
    known_files: &mut std::collections::BTreeMap<PathBuf, crate::sql::FileMetadata>,
    connection: &mut rusqlite::Connection,
) -> ControlFlow<()> {
    let mut meta_data = crate::sql::VideoMetadata {
        ..Default::default()
    };
    let mut nfo_file = PathBuf::from(&format!("{}.nfo", video));
    for fp in all.iter() {
        let f = osstr_to_string(fp.clone().into_os_string());
        //if re.is_match(&f) {
        if f.contains(&video) {
            // part of a local video or metadata
            if f.contains("poster.") {
                meta_data.poster = f.clone();
            } else if f.contains(".srt") {
                meta_data.subtitles.push(f.clone());
            } else if f.contains(".nfo") {
                nfo_file = fp.clone();
            } else if f.ends_with(".mkv") || f.ends_with(".mp4") || f.ends_with(".webm") {
                meta_data.path = osstr_to_string(fp.clone().into_os_string());
            }
            special_files.insert(fp.clone());
        }
    }
    if meta_data.path.len() == 0 {
        return ControlFlow::Break(());
    }
    if !nfo_file.exists() {
        return ControlFlow::Break(());
    }
    
    if meta_data.poster.len() > 0 {
        let thumb = PathBuf::from(&meta_data.poster);
        if thumb.exists() {
            let poster = crate::thumbnails::create_thumbnail(&thumb, 256);
            if poster.len() > 0 {
                meta_data.poster = poster.clone();
            }
        }
    }
    let statdata = match std::fs::metadata(&meta_data.path) {
        Ok(ok) => ok,
        Err(err) => {
            log::warn!(
                "failed to read metadata for entry at {:?}: {}",
                meta_data.path,
                err
            );
            return ControlFlow::Break(());
        }
    };
    items.push(crate::parsers::item_from_nfo(
        nfo_file,
        &mut meta_data,
        &statdata,
        sizes,
        known_files,
        connection,
    ));
    ControlFlow::Continue(())
}

