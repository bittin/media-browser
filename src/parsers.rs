use crate::tab::{Item, ItemMetadata, ItemThumbnail, Location, hidden_attribute};
use crate::mime_icon::{mime_for_path, mime_icon};
use crate::mime_app::mime_apps;
use crate::config::IconSizes;

use chrono::NaiveDate; 
use cosmic::widget::{
        self, Widget,
    };
use mime_guess::mime;
use std::cell::Cell;
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
            Ok(e) => {
                match e {
                    XmlEvent::StartDocument { version, encoding, .. } => {
                        println!("StartDocument({version}, {encoding})");
                    },
                    XmlEvent::EndDocument => {
                        println!("EndDocument");
                        break;
                    },
                    XmlEvent::ProcessingInstruction { name, data } => {
                        println!("ProcessingInstruction({name}={:?})", data.as_deref().unwrap_or_default());
                    },
                    XmlEvent::StartElement { name, attributes, .. } => {
                        tag = name.to_string().to_ascii_lowercase();
                        level += 1;
                        if attributes.is_empty() {
                            println!("StartElement({name})");
                        } else {
                            let attrs: Vec<_> = attributes
                                .iter()
                                .map(|a| format!("{}={:?}", &a.name, a.value))
                                .collect();
                            println!("StartElement({name} [{}])", attrs.join(", "));
                        }
                    },
                    XmlEvent::EndElement { name } => {
                        println!("EndElement({name})");
                        level -= 1;
                    },
                    XmlEvent::Comment(data) => {
                        println!(r#"Comment("{}")"#, data.escape_debug());
                    },
                    XmlEvent::CData(data) => println!(r#"CData("{}")"#, data.escape_debug()),
                    XmlEvent::Characters(data) => {
                        let value = data.escape_debug().to_string();
                        match &tag as &str {
                            "title" => {
                                metadata.title = value.clone();
                            },
                            "plot" => {
                                metadata.description = value.clone();
                            },
                            "runtime" => {
                                let duration = match u32::from_str_radix(&value, 10) {
                                    Ok(ok) => ok,
                                    Err(err) => {
                                        log::warn!("failed to parse number {:?}: {}", value, err);
                                        continue;
                                    }
                                };
                                metadata.duration = duration * 60;
                            },
                            "premiered" => {
                                let ret = NaiveDate::parse_from_str(&value, "%Y-%m-%d");
                                if ret.is_ok() {
                                    metadata.date = ret.unwrap();
                                }
                            },
                            "director" => {
                                metadata.director.push(value.clone());
                            },
                            "actor" => {
                                prevtag = tag.clone();
                            },
                            "name" => {
                                if &prevtag == "actor" {
                                    metadata.actors.push(value.clone());
                                    prevtag.clear();
                                }
                            },
                            "video" => {
                                prevtag = tag.clone();
                            },
                            "width" => {
                                if &prevtag == "video" {
                                    let val = match u32::from_str_radix(&value, 10) {
                                        Ok(ok) => ok,
                                        Err(err) => {
                                            log::warn!("failed to parse number {:?}: {}", value, err);
                                            continue;
                                        }
                                    };
                                    metadata.width = val;
                                }
                            },
                            "height" => {
                                if &prevtag == "video" {
                                    let val = match u32::from_str_radix(&value, 10) {
                                        Ok(ok) => ok,
                                        Err(err) => {
                                            log::warn!("failed to parse number {:?}: {}", value, err);
                                            continue;
                                        }
                                    };
                                    metadata.width = val;
                                }
                            },
                            "durationinseconds" => {
                                if &prevtag == "video" {
                                    let val = match u32::from_str_radix(&value, 10) {
                                        Ok(ok) => ok,
                                        Err(err) => {
                                            log::warn!("failed to parse number {:?}: {}", value, err);
                                            continue;
                                        }
                                    };
                                    metadata.duration = val;
                                }
                            },
                            "audio" => {
                                prevtag = tag.clone();
                            },
                            "subtitle" => {
                                prevtag = tag.clone();
                            },
                            "language" => {
                                if &prevtag == "audio" {
                                    metadata.audiolangs.push(value.clone());
                                }
                                if &prevtag == "subtitle" {
                                    metadata.sublangs.push(value.clone());
                                }
                           },
                           _ => {},
                        }
                    },
                    _ => {},
                }
            },
            Err(e) => {
                break;
            },
        }
    }
}


fn parse_audiotags(nfo_file: &PathBuf, metadata: &mut crate::sql::AudioMetadata) {
    use audiotags::{Tag, MimeType};

    // using `default()` or `new()` alone so that the metadata format is
    // guessed (from the file extension) (in this case, Id3v2 tag is read)
    let tag = Tag::new().read_from_path("test.mp3").unwrap();

    match tag.title() {
        Some(value) => metadata.title = value.to_string(),
        None => {}
    };
    match tag.artists() {
        Some(value) => {
            for s in value {
                metadata.artist.push(s.to_string());
            }
        },
        None => {}
    };
    match tag.album_artists() {
        Some(value) => {
            for s in value {
                metadata.albumartist.push(s.to_string());
            }
        },
        None => {}
    };
    match tag.date() {
        Some(timestamp) => {
            if timestamp.month.is_some() && timestamp.day.is_some() {
                let nd= NaiveDate::from_ymd_opt(
                    timestamp.year, timestamp.month.unwrap() as u32, timestamp.day.unwrap() as u32);
                if nd.is_some() {
                    metadata.date = nd.unwrap();
                }
            } else {
                let nd= NaiveDate::from_ymd_opt(timestamp.year, 1,1);
                if nd.is_some() {
                    metadata.date = nd.unwrap();
                }
            }
        },
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
                        match image::load_from_memory_with_format(picture.data, image::ImageFormat::Jpeg) {
                            Ok(buf) => {
                                let _ = buf.save_with_format(&thumbpath, image::ImageFormat::Png);
                            },
                            Err(error) => {
                                log::warn!("failed to read audio album art jpeg: {}", error);
                            }
                        }
                    },
                    MimeType::Png => {
                        match image::load_from_memory_with_format(picture.data, image::ImageFormat::Png) {
                            Ok(buf) => {
                                let _ = buf.save_with_format(&thumbpath, image::ImageFormat::Png);
                            },
                            Err(error) => {
                                log::warn!("failed to read audio album art jpeg: {}", error);
                            }
                        }
                    },
                    MimeType::Bmp => {
                        match image::load_from_memory_with_format(picture.data, image::ImageFormat::Jpeg) {
                            Ok(buf) => {
                                let _ = buf.save_with_format(&thumbpath, image::ImageFormat::Bmp);
                            },
                            Err(error) => {
                                log::warn!("failed to read audio album art jpeg: {}", error);
                            }
                        }
                    },   
                    MimeType::Gif => {
                        match image::load_from_memory_with_format(picture.data, image::ImageFormat::Gif) {
                            Ok(buf) => {
                                let _ = buf.save_with_format(&thumbpath, image::ImageFormat::Png);
                            },
                            Err(error) => {
                                log::warn!("failed to read audio album art jpeg: {}", error);
                            }
                        }
                    },   
                    MimeType::Tiff => {
                        match image::load_from_memory_with_format(picture.data, image::ImageFormat::Tiff) {
                            Ok(buf) => {
                                let _ = buf.save_with_format(&thumbpath, image::ImageFormat::Png);
                            },
                            Err(error) => {
                                log::warn!("failed to read audio album art jpeg: {}", error);
                            }
                        }
                    },   
                }
            }
            metadata.poster = thumbpath;
        },
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

pub fn item_from_nfo(
    nfo_file: PathBuf,
    metadata: &mut crate::sql::VideoMetadata,
    statdata: &std::fs::Metadata,
    sizes: IconSizes,
    known_files: &mut std::collections::BTreeMap<PathBuf, crate::sql::FileMetadata>,
    connection: &mut rusqlite::Connection,
) -> Item {
    let filepath = PathBuf::from(&metadata.path);
    if known_files.contains_key(&filepath) {
        if let Ok(modified) = statdata.modified() {
            let stored_time = UNIX_EPOCH + Duration::from_secs(known_files[&filepath].modification_time);
            if modified > stored_time {
                // file is newer
                parse_nfo(&nfo_file, metadata);
                crate::sql::update_video(connection, metadata, statdata, known_files);
            } else {
                crate::sql::video(connection, &metadata.path, known_files);
            }
        } else {
            parse_nfo(&nfo_file, metadata);
            crate::sql::insert_video(connection, metadata, statdata, known_files);
        }
    }
    
    let name;
    if metadata.title.len() == 0 {
        name = metadata.name.clone();
    } else {
        name = metadata.title.clone();
    }
    let display_name = Item::display_name(&name);

    let hidden = name.starts_with(".") || hidden_attribute(&statdata);

    let (mime, icon_handle_grid, icon_handle_list, icon_handle_list_condensed) =
        {
            let mime = mime_for_path(PathBuf::from(&metadata.path));
            let thumbpath;
            if metadata.poster.len() == 0 {
                // generate thumbnail
                thumbpath = PathBuf::from("/dummy.png");
            } else {
                thumbpath = PathBuf::from(&metadata.poster);
            }
            (
                mime.clone(),
                widget::icon::from_path(thumbpath.clone()),
                widget::icon::from_path(thumbpath.clone()),
                widget::icon::from_path(thumbpath.clone())
            )
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
        metadata: ItemMetadata::Path { metadata: statdata.clone(), children },
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
    if known_files.contains_key(&filepath) {
        if let Ok(modified) = statdata.modified() {
            let stored_time = UNIX_EPOCH + Duration::from_secs(known_files[&filepath].modification_time);
            if modified > stored_time {
                // file is newer
                parse_audiotags(&nfo_file, metadata);
                crate::sql::update_audio(connection, metadata, statdata, known_files);
            } else {
                crate::sql::audio(connection, &metadata.path, known_files);
            }
        } else {
            parse_audiotags(&nfo_file, metadata);
            crate::sql::insert_audio(connection, metadata, statdata, known_files);
        }
    }
    
    let name;
    if metadata.title.len() == 0 {
        name = metadata.name.clone();
    } else {
        name = metadata.title.clone();
    }
    let display_name = Item::display_name(&name);

    let hidden = name.starts_with(".") || hidden_attribute(&statdata);

    let (mime, icon_handle_grid, icon_handle_list, icon_handle_list_condensed) =
        {
            let mime = mime_for_path(PathBuf::from(&metadata.path));
            let thumbpath;
            if metadata.poster.len() == 0 {
                // generate thumbnail
                thumbpath = PathBuf::from("/dummy.png");
            } else {
                thumbpath = PathBuf::from(&metadata.poster);
            }
            (
                mime.clone(),
                widget::icon::from_path(thumbpath.clone()),
                widget::icon::from_path(thumbpath.clone()),
                widget::icon::from_path(thumbpath.clone())
            )
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
        metadata: ItemMetadata::Path { metadata: statdata.clone(), children },
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
        None => {},
    }
    String::new()
}

