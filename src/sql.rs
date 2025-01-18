// Copyright 2023 System76 <info@system76.com>
// SPDX-License-Identifier: GPL-3.0-only
// 
// Modifications:
// Copyright 2024 Alexander Schwarzkopf

use rusqlite::{Connection, Result, params};
use std::path::PathBuf;
use std::collections::BTreeMap;
use chrono::NaiveDate;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Hash)]
pub struct SearchData {
    pub search_id: u32,
    pub search_string: String,
    pub from_string: String,
    pub from_value_string: String,
    pub from_value: u32,
    pub to_string: String,
    pub to_value_string: String,
    pub to_value: u32,
    pub image: bool,
    pub video: bool,
    pub audio: bool,
    pub filepath: bool,
    pub title: bool,
    pub description: bool,
    pub actor: bool,
    pub director: bool,
    pub artist: bool,
    pub album_artist: bool,
    pub duration: bool,
    pub creation_date: bool,
    pub modification_date: bool,
    pub release_date: bool,
    pub lense_model: bool,
    pub focal_length: bool,
    pub exposure_time: bool,
    pub fnumber: bool,
    pub gps_latitude: bool,
    pub gps_longitude: bool,
    pub gps_altitude: bool,
}

impl Default for SearchData {
    fn default() -> SearchData {
        SearchData {
            search_id: 0,
            search_string: String::new(),
            from_string: String::new(),
            from_value_string: String::new(),
            from_value: 0,
            to_string: String::new(),
            to_value_string: String::new(),
            to_value: 0,
            image: false,
            video: false,
            audio: false,
            filepath: false,
            title: true,
            description: false,
            actor: false,
            director: false,
            artist: false,
            album_artist: false,
            duration: false,
            creation_date: false,
            modification_date: false,
            release_date: false,
            lense_model: false,
            focal_length: false,
            exposure_time: false,
            fnumber: false,
            gps_latitude: false,
            gps_longitude: false,
            gps_altitude: false,
        }
    }
}

impl std::fmt::Display for SearchData {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let _ = write!(f, "{}", self.search_id);
        if self.image {
            let _ = write!(f, " Image");
        } 
        if self.video {
            let _ = write!(f, " Video");
        }
        if self.audio {
            let _ = write!(f, " Audio");
        }
        if self.search_string.len() > 0 {
            let _ = write!(f, " {}", self.search_string);
        }
        if self.from_string.len() > 0 {
            let _ = write!(f, " from {}", self.from_string);
        }
        if self.to_string.len() > 0 {
            let _ = write!(f, " to {}", self.to_string);
        }
        if self.from_value > 0 {
            let _ = write!(f, " from {}", self.from_value);
        }
        if self.to_value > 0 {
            let _ = write!(f, " to {}", self.to_value);
        }
        write!(f, " ")
    }
}

fn search_video_metadata(
    connection: &mut rusqlite::Connection, 
    query: String,
) -> (Vec<VideoMetadata>, Vec<FileMetadata>) {
    let mut files = Vec::new();
    let mut videos = Vec::new();
    let mut ids = Vec::new();
    let mut video_id;

    match connection.prepare(&query) {
        Ok(mut statement) => {
            match statement.query(params![]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                match row.get(0) {
                                    Ok(val) => {
                                        video_id = val;
                                        ids.push(video_id);
                                    },
                                    Err(error) => {
                                        log::error!("Failed to read id for video: {}", error);
                                        continue;
                                    }
                                }
                                
                            },
                            Ok(None) => {
                                //log::warn!("No data read from indices.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from indices: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from videostore_indices database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }
    for video_id in ids {
        let filedata = file_by_id(connection, video_id);
        let pathstr = crate::parsers::osstr_to_string(filedata.filepath.clone().into_os_string());
        let videodata = video_by_id(connection, &pathstr, video_id);
        files.push(filedata);
        videos.push(videodata);
    }
    (videos, files)
}

pub fn search_video(
    connection: &mut rusqlite::Connection, 
    search: &SearchData,
) -> (Vec<VideoMetadata>, Vec<FileMetadata>) {
    let mut files: Vec<FileMetadata> = Vec::new();
    let mut videos: Vec<VideoMetadata> = Vec::new();
    let mut used_files: std::collections::BTreeSet<PathBuf> = std::collections::BTreeSet::new();
    if !search.video {
        return (videos, files);
    }
    if search.title {
        let query = format!("SELECT video_id FROM video_metadata WHERE title LIKE '%{}%'", search.search_string);
        let (newvideos, newfiles) = search_video_metadata(
            connection, 
            query, 
        );
        for i in 0..newfiles.len() {
            if !used_files.contains(&newfiles[i].filepath) {
                used_files.insert(newfiles[i].filepath.clone());
                files.push(newfiles[i].clone());
                videos.push(newvideos[i].clone());
            }
        }
    }
    if search.description {
        let query = format!("SELECT video_id FROM video_metadata WHERE description LIKE '%{}%'", search.search_string);
        let (newvideos, newfiles) = search_video_metadata(
            connection, 
            query, 
        );
        for i in 0..newfiles.len() {
            if !used_files.contains(&newfiles[i].filepath) {
                used_files.insert(newfiles[i].filepath.clone());
                files.push(newfiles[i].clone());
                videos.push(newvideos[i].clone());
            }
        }
    }
    if search.duration && search.from_value != 0 {
        let query;
        if search.to_value != 0 {
            query = format!("SELECT video_id FROM video_metadata WHERE duration > {} AND duration < {}", search.from_value as u32, search.to_value as u32);
        } else {
            query = format!("SELECT video_id FROM video_metadata WHERE duration > {}", search.from_value as u32);
        }
        let (newvideos, newfiles) = search_video_metadata(
            connection, 
            query, 
        );
        for i in 0..newfiles.len() {
            if !used_files.contains(&newfiles[i].filepath) {
                used_files.insert(newfiles[i].filepath.clone());
                files.push(newfiles[i].clone());
                videos.push(newvideos[i].clone());
            }
        }
    }
    if search.actor {
        let query = format!("SELECT video_id FROM actors INNER JOIN people ON people.person_id = actors.actor_id WHERE people.person_name LIKE '%{}%'", search.search_string);
        let (newvideos, newfiles) = search_video_metadata(
            connection, 
            query, 
        );
        for i in 0..newfiles.len() {
            if !used_files.contains(&newfiles[i].filepath) {
                used_files.insert(newfiles[i].filepath.clone());
                files.push(newfiles[i].clone());
                videos.push(newvideos[i].clone());
            }
        }
    }
    if search.director {
        let query = format!("SELECT video_id FROM directors INNER JOIN people ON people.person_id = directors.director_id WHERE people.person_name LIKE '%{}%'", search.search_string);
        let (newvideos, newfiles) = search_video_metadata(
            connection, 
            query, 
        );
        for i in 0..newfiles.len() {
            if !used_files.contains(&newfiles[i].filepath) {
                used_files.insert(newfiles[i].filepath.clone());
                files.push(newfiles[i].clone());
                videos.push(newvideos[i].clone());
            }
        }
    }
    if search.release_date && search.from_string.len() != 0 {
        let from_date = string_to_linux_time(&search.from_string);
        let to_date = string_to_linux_time(&search.to_string);
        let query;
        if to_date != 0 {
            query = format!("SELECT video_id FROM video_metadata WHERE released > {} AND released < {}", from_date, to_date);
        } else {
            query = format!("SELECT video_id FROM video_metadata WHERE released > {}", from_date);
        }
        let (newvideos, newfiles) = search_video_metadata(
            connection, 
            query, 
        );
        for i in 0..newfiles.len() {
            if !used_files.contains(&newfiles[i].filepath) {
                used_files.insert(newfiles[i].filepath.clone());
                files.push(newfiles[i].clone());
                videos.push(newvideos[i].clone());
            }
        }
    }
    
    (videos, files)
}

fn search_audio_metadata(
    connection: &mut rusqlite::Connection, 
    query: String,
) -> (Vec<AudioMetadata>, Vec<FileMetadata>) {
    let mut files = Vec::new();
    let mut audios = Vec::new();
    let mut ids = Vec::new();
    let mut audio_id;

    match connection.prepare(&query) {
        Ok(mut statement) => {
            match statement.query(params![]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                match row.get(0) {
                                    Ok(val) => {
                                        audio_id = val;
                                        ids.push(audio_id);
                                    },
                                    Err(error) => {
                                        log::error!("Failed to read id for video: {}", error);
                                        continue;
                                    }
                                }
                                
                            },
                            Ok(None) => {
                                //log::warn!("No data read from indices.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from indices: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from videostore_indices database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }
    for audio_id in ids {
        let filedata = file_by_id(connection, audio_id);
        let pathstr = crate::parsers::osstr_to_string(filedata.filepath.clone().into_os_string());
        let audiodata = audio_by_id(connection, &pathstr, audio_id);
        files.push(filedata);
        audios.push(audiodata);
    }
    (audios, files)
}

pub fn search_audio(
    connection: &mut rusqlite::Connection, 
    search: &SearchData,
) -> (Vec<AudioMetadata>, Vec<FileMetadata>) {
    let mut files: Vec<FileMetadata> = Vec::new();
    let mut audios: Vec<AudioMetadata> = Vec::new();
    let mut used_files: std::collections::BTreeSet<PathBuf> = std::collections::BTreeSet::new();
    if !search.audio {
        return (audios, files);
    }
    if search.title {
        let query = format!("SELECT audio_id FROM audio_metadata WHERE title LIKE '%{}%'", search.search_string);
        let (newvideos, newfiles) = search_audio_metadata(
            connection, 
            query, 
        );
        for i in 0..newfiles.len() {
            if !used_files.contains(&newfiles[i].filepath) {
                used_files.insert(newfiles[i].filepath.clone());
                files.push(newfiles[i].clone());
                audios.push(newvideos[i].clone());
            }
        }
    }
    if search.duration && search.from_value != 0 {
        let query;
        if search.to_value != 0 {
            query = format!("SELECT audio_id FROM audio_metadata WHERE duration > {} AND duration < {}", search.from_value / 1000000, search.to_value / 1000000);
        } else {
            query = format!("SELECT audio_id FROM audio_metadata WHERE duration > {}", search.from_value  / 1000000);
        }
        let (newvideos, newfiles) = search_audio_metadata(
            connection, 
            query, 
        );
        for i in 0..newfiles.len() {
            if !used_files.contains(&newfiles[i].filepath) {
                used_files.insert(newfiles[i].filepath.clone());
                files.push(newfiles[i].clone());
                audios.push(newvideos[i].clone());
            }
        }
    }
    if search.artist {
        let query = format!("SELECT audio_id FROM artist_audio_map INNER JOIN artists ON artists.artist_id  = artist_audio_map.artist_id WHERE artists.artist_name LIKE '%{}%'", search.search_string);
        let (newvideos, newfiles) = search_audio_metadata(
            connection, 
            query, 
        );
        for i in 0..newfiles.len() {
            if !used_files.contains(&newfiles[i].filepath) {
                used_files.insert(newfiles[i].filepath.clone());
                files.push(newfiles[i].clone());
                audios.push(newvideos[i].clone());
            }
        }
    }
    if search.album_artist {
        let query = format!("SELECT audio_id FROM albumartist_audio_map INNER JOIN artists ON artists.artist_id  = albumartist_audio_map.artist_id WHERE artists.artist_name LIKE '%{}%'", search.search_string);
        let (newvideos, newfiles) = search_audio_metadata(
            connection, 
            query, 
        );
        for i in 0..newfiles.len() {
            if !used_files.contains(&newfiles[i].filepath) {
                used_files.insert(newfiles[i].filepath.clone());
                files.push(newfiles[i].clone());
                audios.push(newvideos[i].clone());
            }
        }
    }
    if search.release_date && search.from_string.len() != 0 {
        let from_date = string_to_linux_time(&search.from_string);
        let to_date = string_to_linux_time(&search.to_string);
        let query;
        if to_date != 0 {
            query = format!("SELECT audio_id FROM audio_metadata WHERE released > {} AND released < {}", from_date, to_date);
        } else {
            query = format!("SELECT audio_id FROM audio_metadata WHERE released > {}", from_date);
        }
        let (newvideos, newfiles) = search_audio_metadata(
            connection, 
            query, 
        );
        for i in 0..newfiles.len() {
            if !used_files.contains(&newfiles[i].filepath) {
                used_files.insert(newfiles[i].filepath.clone());
                files.push(newfiles[i].clone());
                audios.push(newvideos[i].clone());
            }
        }
    }
    
    (audios, files)
}

fn search_image_metadata(
    connection: &mut rusqlite::Connection, 
    query: String,
) -> (Vec<ImageMetadata>, Vec<FileMetadata>) {
    let mut files = Vec::new();
    let mut images = Vec::new();
    let mut ids = Vec::new();
    let mut image_id;

    match connection.prepare(&query) {
        Ok(mut statement) => {
            match statement.query(params![]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                match row.get(0) {
                                    Ok(val) => {
                                        image_id = val;
                                        ids.push(image_id);
                                    },
                                    Err(error) => {
                                        log::error!("Failed to read id for video: {}", error);
                                        continue;
                                    }
                                }
                                
                            },
                            Ok(None) => {
                                //log::warn!("No data read from indices.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from indices: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from videostore_indices database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }
    for image_id in ids {
        let filedata = file_by_id(connection, image_id);
        let pathstr = crate::parsers::osstr_to_string(filedata.filepath.clone().into_os_string());
        let imagedata = image_by_id(connection, &pathstr, image_id);
        files.push(filedata);
        images.push(imagedata);
    }
    (images, files)
}

pub fn search_image(
    connection: &mut rusqlite::Connection, 
    search: &SearchData,
) -> (Vec<ImageMetadata>, Vec<FileMetadata>) {
    let mut files = Vec::new();
    let mut images: Vec<ImageMetadata> = Vec::new();

    let mut used_files: std::collections::BTreeSet<PathBuf> = std::collections::BTreeSet::new();
    if !search.audio {
        return (images, files);
    }
    if search.title {
        let query = format!("SELECT image_id FROM image_metadata WHERE name LIKE '%{}%'", search.search_string);
        let (newvideos, newfiles) = search_image_metadata(
            connection, 
            query, 
        );
        for i in 0..newfiles.len() {
            if !used_files.contains(&newfiles[i].filepath) {
                used_files.insert(newfiles[i].filepath.clone());
                files.push(newfiles[i].clone());
                images.push(newvideos[i].clone());
            }
        }
    }
    if search.lense_model {
        let query = format!("SELECT image_id FROM image_metadata WHERE LenseModel LIKE '%{}%'", search.search_string);
        let (newvideos, newfiles) = search_image_metadata(
            connection, 
            query, 
        );
        for i in 0..newfiles.len() {
            if !used_files.contains(&newfiles[i].filepath) {
                used_files.insert(newfiles[i].filepath.clone());
                files.push(newfiles[i].clone());
                images.push(newvideos[i].clone());
            }
        }
    }
    if search.focal_length {
        let query = format!("SELECT image_id FROM image_metadata WHERE Focallength LIKE '%{}%'", search.search_string);
        let (newvideos, newfiles) = search_image_metadata(
            connection, 
            query, 
        );
        for i in 0..newfiles.len() {
            if !used_files.contains(&newfiles[i].filepath) {
                used_files.insert(newfiles[i].filepath.clone());
                files.push(newfiles[i].clone());
                images.push(newvideos[i].clone());
            }
        }
    }
    if search.exposure_time {
        let query = format!("SELECT image_id FROM image_metadata WHERE Exposuretime LIKE '%{}%'", search.search_string);
        let (newvideos, newfiles) = search_image_metadata(
            connection, 
            query, 
        );
        for i in 0..newfiles.len() {
            if !used_files.contains(&newfiles[i].filepath) {
                used_files.insert(newfiles[i].filepath.clone());
                files.push(newfiles[i].clone());
                images.push(newvideos[i].clone());
            }
        }
    }
    if search.fnumber {
        let query = format!("SELECT image_id FROM image_metadata WHERE FNumber LIKE '%{}%'", search.search_string);
        let (newvideos, newfiles) = search_image_metadata(
            connection, 
            query, 
        );
        for i in 0..newfiles.len() {
            if !used_files.contains(&newfiles[i].filepath) {
                used_files.insert(newfiles[i].filepath.clone());
                files.push(newfiles[i].clone());
                images.push(newvideos[i].clone());
            }
        }
    }
    if search.gps_latitude && search.from_value != 0 {
        let query;
        if search.to_value != 0 {
            query = format!("SELECT image_id FROM image_metadata WHERE GPSLatitude > {} AND GPSLatitude < {}", search.from_value / 1000000, search.to_value / 1000000);
        } else {
            query = format!("SELECT image_id FROM image_metadata WHERE GPSLatitude > {}", search.from_value / 1000000);
        }
        let (newvideos, newfiles) = search_image_metadata(
            connection, 
            query, 
        );
        for i in 0..newfiles.len() {
            if !used_files.contains(&newfiles[i].filepath) {
                used_files.insert(newfiles[i].filepath.clone());
                files.push(newfiles[i].clone());
                images.push(newvideos[i].clone());
            }
        }
    }
    if search.gps_longitude && search.from_value != 0 {
        let query;
        if search.to_value != 0 {
            query = format!("SELECT image_id FROM image_metadata WHERE GPSLongitude > {} AND GPSLongitude < {}", search.from_value / 1000000, search.to_value / 1000000);
        } else {
            query = format!("SELECT image_id FROM image_metadata WHERE GPSLongitude > {}", search.from_value / 1000000);
        }
        let (newvideos, newfiles) = search_image_metadata(
            connection, 
            query, 
        );
        for i in 0..newfiles.len() {
            if !used_files.contains(&newfiles[i].filepath) {
                used_files.insert(newfiles[i].filepath.clone());
                files.push(newfiles[i].clone());
                images.push(newvideos[i].clone());
            }
        }
    }
    if search.gps_altitude && search.from_value != 0 {
        let query;
        if search.to_value != 0 {
            query = format!("SELECT image_id FROM image_metadata WHERE GPSAltitude > {} AND GPSAltitude < {}", search.from_value / 1000000, search.to_value / 1000000);
        } else {
            query = format!("SELECT image_id FROM image_metadata WHERE GPSAltitude > {}", search.from_value / 1000000);
        }
        let (newvideos, newfiles) = search_image_metadata(
            connection, 
            query, 
        );
        for i in 0..newfiles.len() {
            if !used_files.contains(&newfiles[i].filepath) {
                used_files.insert(newfiles[i].filepath.clone());
                files.push(newfiles[i].clone());
                images.push(newvideos[i].clone());
            }
        }
    }
    if search.release_date && search.from_string.len() != 0 {
        let from_date = string_to_linux_time(&search.from_string);
        let to_date = string_to_linux_time(&search.to_string);
        let query;
        if to_date != 0 {
            query = format!("SELECT image_id FROM image_metadata WHERE created > {} AND created < {}", from_date, to_date);
        } else {
            query = format!("SELECT image_id FROM image_metadata WHERE created > {}", from_date);
        }
        let (newvideos, newfiles) = search_image_metadata(
            connection, 
            query, 
        );
        for i in 0..newfiles.len() {
            if !used_files.contains(&newfiles[i].filepath) {
                used_files.insert(newfiles[i].filepath.clone());
                files.push(newfiles[i].clone());
                images.push(newvideos[i].clone());
            }
        }
    }
    
    (images, files)
}

fn search_file_metadata(
    connection: &mut rusqlite::Connection, 
    query: String,
) -> Vec<FileMetadata> {
    let mut files = Vec::new();
    let mut ids = Vec::new();
    let mut file_id;

    match connection.prepare(&query) {
        Ok(mut statement) => {
            match statement.query(params![]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                match row.get(0) {
                                    Ok(val) => {
                                        file_id = val;
                                        ids.push(file_id);
                                    },
                                    Err(error) => {
                                        log::error!("Failed to read id for video: {}", error);
                                        continue;
                                    }
                                }
                                
                            },
                            Ok(None) => {
                                //log::warn!("No data read from indices.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from indices: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from videostore_indices database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }
    for file_id in ids {
        let filedata = file_by_id(connection, file_id);
        files.push(filedata);
    }
    files
}

fn stuff_items(
    connection: &mut rusqlite::Connection, 
    search: &SearchData,
    known_files: &mut std::collections::BTreeMap<PathBuf, FileMetadata>,
    used_files: &mut std::collections::BTreeSet<PathBuf>,
    file: FileMetadata,
    items: &mut Vec<crate::tab::Item>,
) {
    if !used_files.contains(&file.filepath) {
        used_files.insert(file.filepath.clone());
        if file.file_type == 2 && search.video {
            let pathstr = crate::parsers::osstr_to_string(file.filepath.clone().into_os_string());
            let mut video = video(connection, &pathstr, known_files);
            if let Ok(metadata) = std::fs::metadata(file.filepath.clone()) {
                let item = crate::parsers::item_from_nfo(
                    PathBuf::new(), 
                    &mut video, 
                    &metadata, 
                    crate::config::IconSizes::default(), 
                    known_files, 
                    connection,
                    true,
                );
                items.push(item);
            }    
        }
        if file.file_type == 3 && search.audio {
            let pathstr = crate::parsers::osstr_to_string(file.filepath.clone().into_os_string());
            let mut audio = audio(connection, &pathstr, known_files);
            let mut special_files = std::collections::BTreeSet::new();
            if let Ok(metadata) = std::fs::metadata(file.filepath.clone()) {
                let item = crate::parsers::item_from_audiotags(
                    file.filepath.clone(),
                    &mut special_files,
                    &mut audio, 
                    &metadata, 
                    crate::config::IconSizes::default(), 
                    known_files, 
                    connection,
                    true,
                );
                items.push(item);
            }    

        }
        if file.file_type == 1 && search.image {
            let pathstr = crate::parsers::osstr_to_string(file.filepath.clone().into_os_string());
            let mut image = image(connection, &pathstr, known_files);
            if let Ok(metadata) = std::fs::metadata(file.filepath.clone()) {
                let item = crate::parsers::item_from_exif(
                    file.filepath.clone(), 
                    &mut image, 
                    &metadata, 
                    crate::config::IconSizes::default(), 
                    known_files, 
                    connection,
                    true
                );
                items.push(item);
            }                
        }
    }
}

/// date has to be in the following format
/// YYYY-MM-DDThh:mm:ss (1996-12-19T16:39:57)
fn string_to_linux_time(date: &str) -> i64 {
    let mut linuxtime = 0;
    if let Ok(date) = chrono::DateTime::parse_from_rfc3339(date) {
        let naive = date.naive_utc();
        linuxtime = naive.and_utc().timestamp();
    }

    linuxtime
}

pub fn search_items(
    connection: &mut rusqlite::Connection, 
    search: &SearchData,
) -> Vec<crate::tab::Item> {
    let mut items: Vec<crate::tab::Item> = Vec::new();
    let mut used_files: std::collections::BTreeSet<PathBuf> = std::collections::BTreeSet::new();
    let mut known_files: std::collections::BTreeMap<PathBuf, FileMetadata> = std::collections::BTreeMap::new();
    if search.video {
        let (mut newmetadata, newfiles) = search_video(connection, search);
        for i in 0..newfiles.len() {
            if !used_files.contains(&newfiles[i].filepath) {
                used_files.insert(newfiles[i].filepath.clone());
                if let Ok(metadata) = std::fs::metadata(newfiles[i].filepath.clone()) {
                    let item = crate::parsers::item_from_nfo(
                        PathBuf::new(), 
                        &mut newmetadata[i], 
                        &metadata, 
                        crate::config::IconSizes::default(), 
                        &mut known_files, 
                        connection,
                        true,
                    );
                    items.push(item);
                }
            }
        }
    }
    if search.audio {
        let (mut newmetadata, newfiles) = search_audio(connection, search);
        for i in 0..newfiles.len() {
            if !used_files.contains(&newfiles[i].filepath) {
                used_files.insert(newfiles[i].filepath.clone());
                let mut special_files = std::collections::BTreeSet::new();
                if let Ok(metadata) = std::fs::metadata(newfiles[i].filepath.clone()) {
                    let item = crate::parsers::item_from_audiotags(
                        newfiles[i].filepath.clone(),
                        &mut special_files,
                        &mut newmetadata[i], 
                        &metadata, 
                        crate::config::IconSizes::default(), 
                        &mut known_files, 
                        connection,
                        true
                    );
                    items.push(item);
                }
            }
        }
    }
    if search.image {
        let (mut newmetadata, newfiles) = search_image(connection, search);
        for i in 0..newfiles.len() {
            if !used_files.contains(&newfiles[i].filepath) {
                used_files.insert(newfiles[i].filepath.clone());
                if let Ok(metadata) = std::fs::metadata(newfiles[i].filepath.clone()) {
                    let item = crate::parsers::item_from_exif(
                        newfiles[i].filepath.clone(), 
                        &mut newmetadata[i], 
                        &metadata, 
                        crate::config::IconSizes::default(), 
                        &mut known_files, 
                        connection,
                        true
                    );
                    items.push(item);
                }
            }
        }
    }
    if search.creation_date && search.from_string.len() != 0 {
        let from_date = string_to_linux_time(&search.from_string);
        let to_date = string_to_linux_time(&search.to_string);
        let query;
        if to_date != 0 {
            query = format!("SELECT metadata_id FROM file_metadata WHERE creation_time > {} AND creation_time < {}", from_date, to_date);
        } else {
            query = format!("SELECT metadata_id FROM file_metadata WHERE creation_time > {}", from_date);
        }
        let newfiles: Vec<FileMetadata> = search_file_metadata(
            connection, 
            query, 
        );
        for file in newfiles {
            stuff_items(connection, search, &mut known_files, &mut used_files, file, &mut items);
        }
    }
    if search.modification_date && search.from_string.len() != 0 {
        let from_date = string_to_linux_time(&search.from_string);
        let to_date = string_to_linux_time(&search.to_string);
        let query;
        if to_date != 0 {
            query = format!("SELECT metadata_id FROM file_metadata WHERE modification_time > {} AND modification_time < {}", from_date, to_date);
        } else {
            query = format!("SELECT metadata_id FROM file_metadata WHERE modification_time > {}", from_date);
        }
        let newfiles: Vec<FileMetadata> = search_file_metadata(
            connection, 
            query, 
        );
        for file in newfiles {
            stuff_items(connection, search, &mut known_files, &mut used_files, file, &mut items);
        }
    }
    if search.filepath {
        let query = format!("SELECT metadata_id FROM file_metadata WHERE filepath LIKE '%{}%'", search.search_string);
        let newfiles: Vec<FileMetadata> = search_file_metadata(
            connection, 
            query, 
        );
        for file in newfiles {
            stuff_items(connection, search, &mut known_files, &mut used_files, file, &mut items);
        }
    }

    items
}

#[derive(Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Chapter {
    pub title: String,
    pub start: f32,
    pub end: f32,
}

impl std::fmt::Display for Chapter {
    // This trait requires `fmt` with this exact signature.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} -- {}", self.title, self.start)
    }
}

pub fn fill_chapters(chapters: Vec<crate::sql::Chapter>, duration: u32) -> (Vec<crate::sql::Chapter>, Vec<String>) {
    let mut v = Vec::new();
    let mut s = Vec::new();
    if chapters.len() > 0 {
        v.extend(chapters.clone());
    } else {
        let numstep = duration / 5 / 60;
        let mut i = 0;
        while i < numstep {
            s.push(format!("Chapter{:02}", i));
            i += 1;
        }
    }
    for i in 0..v.len() {
        let title = format!("{:02} {}", i + 1, v[i].title);
        s.push(title);
    }

    (v, s)
}


#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct VideoMetadata {
    pub name: String,
    pub title: String,
    pub date: NaiveDate,
    pub path: String,
    pub poster: String,
    pub thumb: String,
    pub subtitles: Vec<String>,
    pub audiolangs: Vec<String>,
    pub sublangs: Vec<String>,
    pub duration: u32,
    pub width: u32,
    pub height: u32,
    pub framerate: f32,
    pub description: String, 
    pub director: Vec<String>,
    pub actors: Vec<String>,
    pub chapters: Vec<Chapter>,
}

impl Default for VideoMetadata {
    fn default() -> VideoMetadata {
        VideoMetadata {
            name: String::new(),
            title: String::new(),
            date: NaiveDate::from_ymd_opt(1970, 1,1).unwrap(),
            path: String::new(),
            poster: String::new(),
            thumb: String::new(),
            subtitles: Vec::new(),
            audiolangs: Vec::new(),
            sublangs: Vec::new(),

            duration: 0,
            width: 0,
            height: 0,
            framerate: 0.0,
            description: String::new(),
            director: Vec::new(),
            actors: Vec::new(),
            chapters: Vec::new(),
        }
    }
}

pub fn insert_video(
    connection: &mut rusqlite::Connection, 
    metadata: &mut crate::sql::VideoMetadata,
    statdata: &std::fs::Metadata,
    known_files: &mut std::collections::BTreeMap<PathBuf, crate::sql::FileMetadata>,
) {
    match connection.execute(
        "INSERT INTO video_metadata (name, title, released, poster, thumb, duration, width, height, framerate, description) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![&metadata.name, &metadata.title, &metadata.date, &metadata.poster, &metadata.thumb, &metadata.duration, &metadata.width, &metadata.height, &metadata.framerate, &metadata.description],
    ) {
        Ok(_retval) => {}, //log::warn!("Inserted {} video with ID {} and location {} into candidates.", video.id, video.index, candidate_id),
        Err(error) => {
            log::error!("Failed to insert video into  database: {}", error);
            return;
        }
    }
    let mut video_id = 0;
    let query = "SELECT last_insert_rowid()";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![]) {
                Ok(mut rows) => {
                    while let Ok(Some(row)) = rows.next() {
                        let s_opt = row.get(0);
                        if s_opt.is_ok() {
                            video_id = s_opt.unwrap();
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from video_metadata database: {}", err);
                }
            }
        },
        Err(error) => {
            log::error!("Failed to get video_id for from database: {}", error);
            return;
        }
    }
    insert_file(connection, &metadata.path, statdata, 2, video_id, known_files);
    for i in 0..metadata.subtitles.len() {
        match connection.execute(
            "INSERT INTO subtitles (video_id, subpath) VALUES (?1, ?2)",
            params![&video_id, &metadata.subtitles[i]],
        ) {
            Ok(_retval) => {}, //log::warn!("Inserted {} video with ID {} and location {} into candidates.", video.id, video.index, candidate_id),
            Err(error) => {
                log::error!("Failed to insert video into  database: {}", error);
                return;
            }
        }
    }
    for i in 0..metadata.audiolangs.len() {
        match connection.execute(
            "INSERT INTO audiolangs (video_id, audiolang) VALUES (?1, ?2)",
            params![&video_id, &metadata.audiolangs[i]],
        ) {
            Ok(_retval) => {}, //log::warn!("Inserted {} video with ID {} and location {} into candidates.", video.id, video.index, candidate_id),
            Err(error) => {
                log::error!("Failed to insert video into  database: {}", error);
                return;
            }
        }
    }
    for i in 0..metadata.sublangs.len() {
        match connection.execute(
            "INSERT INTO sublangs (video_id, sublang) VALUES (?1, ?2)",
            params![&video_id, &metadata.sublangs[i]],
        ) {
            Ok(_retval) => {}, //log::warn!("Inserted {} video with ID {} and location {} into candidates.", video.id, video.index, candidate_id),
            Err(error) => {
                log::error!("Failed to insert video into  database: {}", error);
                return;
            }
        }
    }
    for i in 0..metadata.chapters.len() {
        match connection.execute(
            "INSERT INTO chapters (video_id, title, start, end) VALUES (?1, ?2, ?3, ?4)",
            params![&video_id, &metadata.chapters[i].title, &metadata.chapters[i].start, &metadata.chapters[i].end],
        ) {
            Ok(_retval) => {}, //log::warn!("Inserted {} video with ID {} and location {} into candidates.", video.id, video.index, candidate_id),
            Err(error) => {
                log::error!("Failed to insert video into  database: {}", error);
                return;
            }
        }
    }
    for i in 0..metadata.director.len() {
        let director_id = insert_person(connection, metadata.director[i].clone());
        if director_id == -1 {
            continue;
        }
        match connection.execute(
            "INSERT INTO directors (director_id, video_id) VALUES (?1, ?2)",
            params![&director_id, &video_id],
        ) {
            Ok(_retval) => {}, //log::warn!("Inserted {} video with ID {} and location {} into candidates.", video.id, video.index, candidate_id),
            Err(error) => {
                log::error!("Failed to insert director into  database: {}", error);
                return;
            }
        }
    }

    for i in 0..metadata.actors.len() {
        let actor_id = insert_person(connection, metadata.actors[i].clone());
        if actor_id == -1 {
            continue;
        }
        match connection.execute(
            "INSERT INTO actors (actor_id, video_id) VALUES (?1, ?2)",
            params![&actor_id, &video_id],
        ) {
            Ok(_retval) => {}, //log::warn!("Inserted {} video with ID {} and location {} into candidates.", video.id, video.index, candidate_id),
            Err(error) => {
                log::error!("Failed to insert actor into  database: {}", error);
                return;
            }
        }
    }

}

fn insert_person(connection: &mut rusqlite::Connection, name: String) -> i32 {
    let mut person_id= -1;
    let query = "SELECT person_id FROM people WHERE person_name = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&name]) {
                Ok(mut rows) => {
                    while let Ok(Some(row)) = rows.next() {
                        let s_opt = row.get(1);
                        if s_opt.is_ok() {
                            person_id = s_opt.unwrap();
                            return person_id;
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from people database: {}", err);
                }
            }
        },
        Err(error) => {
            log::error!("Failed to get person_id for {} from database: {}", name, error);
            return person_id;
        }
    }
    if person_id == -1 {
        match connection.execute(
            "INSERT INTO people (person_name) VALUES (?1)",
            params![&name],
        ) {
            Ok(_retval) => {}, //log::warn!("Inserted {} video with ID {} and location {} into candidates.", video.id, video.index, candidate_id),
            Err(error) => {
                log::error!("Failed to insert video into  database: {}", error);
                return person_id;
            }
        }

        let query = "SELECT last_insert_rowid()";
        match connection.prepare(query) {
            Ok(mut statement) => {
                match statement.query(params![]) {
                    Ok(mut rows) => {
                        while let Ok(Some(row)) = rows.next() {
                            let s_opt = row.get(0);
                            if s_opt.is_ok() {
                                person_id = s_opt.unwrap();
                            }
                        }
                    },
                    Err(err) => {
                        log::error!("could not read last generated id from database: {}", err);
                    }
                }
            },
            Err(error) => {
                log::error!("Failed to get person_id for from database: {}", error);
                return person_id;
            }
        }
    }
    person_id
}

pub fn delete_video(
    connection: &mut rusqlite::Connection, 
    metadata: &mut crate::sql::VideoMetadata,
    known_files: &mut std::collections::BTreeMap<PathBuf, crate::sql::FileMetadata>,
) {
    // Get the index.
    //let index = self.ids[id];
    let mut video_id: u32 = 0;
    let query = "SELECT metadata_id FROM file_metadata WHERE filepath = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&metadata.path]) {
                Ok(mut rows) => {
                    while let Ok(Some(row)) = rows.next() {
                        let s_opt = row.get(1);
                        if s_opt.is_ok() {
                            video_id = s_opt.unwrap();
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from file_metadata database: {}", err);
                }
            }
        },
        Err(error) => {
            log::error!("Failed to get video_id for {} from database: {}", metadata.path, error);
            return;
        }
    }
    // clear the entry in the candidates list without deleting it
    let ret = connection.execute(
        "DELETE FROM video_metadata WHERE video_id = ?1",
        params![&video_id],
    );
    if ret.is_err() {
        log::error!("Failed to delete candidate {}!", video_id);
        return;
    }    
    // clear the entry in the candidates list without deleting it
    let ret = connection.execute(
        "DELETE FROM subtitles WHERE video_id = ?1",
        params![&video_id],
    );
    if ret.is_err() {
        log::error!("Failed to delete subtitles {}!", video_id);
        return;
    }    
    // clear the entry in the candidates list without deleting it
    let ret = connection.execute(
        "DELETE FROM audiolangs WHERE video_id = ?1",
        params![&video_id],
    );
    if ret.is_err() {
        log::error!("Failed to delete candidate {}!", video_id);
        return;
    }    
    // clear the entry in the candidates list without deleting it
    let ret = connection.execute(
        "DELETE FROM sublangs WHERE video_id = ?1",
        params![&video_id],
    );
    if ret.is_err() {
        log::error!("Failed to delete candidate {}!", video_id);
        return;
    }    
    // clear the entry in the candidates list without deleting it
    let ret = connection.execute(
        "DELETE FROM director WHERE video_id = ?1",
        params![&video_id],
    );
    if ret.is_err() {
        log::error!("Failed to delete candidate {}!", video_id);
        return;
    }    
    // clear the entry in the candidates list without deleting it
    let ret = connection.execute(
        "DELETE FROM actors WHERE video_id = ?1",
        params![&video_id],
    );
    if ret.is_err() {
        log::error!("Failed to delete candidate {}!", video_id);
        return;
    }    
    // clear the entry in the candidates list without deleting it
    let ret = connection.execute(
        "DELETE FROM file_metadata WHERE filepath = ?1",
        params![&metadata.path],
    );
    if ret.is_err() {
        log::error!("Failed to delete candidate {}!", metadata.path);
        return;
    }   
    known_files.remove(&PathBuf::from(&metadata.path));
 
}

pub fn update_video(
    connection: &mut rusqlite::Connection, 
    metadata: &mut crate::sql::VideoMetadata,
    statdata: &std::fs::Metadata,
    known_files: &mut std::collections::BTreeMap<PathBuf, crate::sql::FileMetadata>,
) {
    delete_video(connection, metadata, known_files);
    insert_video(connection, metadata, statdata, known_files);
}

pub fn video_by_id(
    connection: &mut rusqlite::Connection, 
    filepath: &str,
    video_id: i64,
) -> VideoMetadata {
    let mut v = VideoMetadata {..Default::default()};
    v.path = filepath.to_string();
    let query = "SELECT name, title, released, poster, duration, width, height, framerate, description, thumb FROM video_metadata WHERE video_id = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&video_id]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                match row.get(0) {
                                    Ok(val) => v.name = val,
                                    Err(error) => {
                                        log::error!("Failed to read id for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(1) {
                                    Ok(val) => v.title = val,
                                    Err(error) => {
                                        log::error!("Failed to read video_id for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(2) {
                                    Ok(val) => v.date = val,
                                    Err(error) => {
                                        log::error!("Failed to read screenshot_id for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(3) {
                                    Ok(val) => v.poster = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(4) {
                                    Ok(val) => v.duration = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(5) {
                                    Ok(val) => v.width = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(6) {
                                    Ok(val) => v.height = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(7) {
                                    Ok(val) => v.framerate = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(8) {
                                    Ok(val) => v.description = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(9) {
                                    Ok(val) => v.thumb = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                            },
                            Ok(None) => {
                                //log::warn!("No data read from indices.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from indices: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from videostore_indices database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }
    let query = "SELECT subpath FROM subtitles WHERE video_id = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&video_id]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                match row.get::<usize, String>(0) {
                                    Ok(val) => {
                                        let s: String = val;
                                        v.subtitles.push(s);
                                    },
                                    Err(error) => {
                                        log::error!("Failed to read id for video: {}", error);
                                        continue;
                                    }
                                }
                            }
                            Ok(None) => {
                                //log::warn!("No data read from indices.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from indices: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from subtitles database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }
    let query = "SELECT audiolang FROM audiolangs WHERE video_id = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&video_id]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                match row.get::<usize, String>(0) {
                                    Ok(val) => v.audiolangs.push(val.clone()),
                                    Err(error) => {
                                        log::error!("Failed to audiolangs for video: {}", error);
                                        continue;
                                    }
                                }
                            }
                            Ok(None) => {
                                //log::warn!("No data read from indices.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from audiolangs: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from audiolangs database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }
    let query = "SELECT sublang FROM sublangs WHERE video_id = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&video_id]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                match row.get::<usize, String>(0) {
                                    Ok(val) => v.sublangs.push(val.clone()),
                                    Err(error) => {
                                        log::error!("Failed to read sublangs for video: {}", error);
                                        continue;
                                    }
                                }
                            }
                            Ok(None) => {
                                //log::warn!("No data read from sublangs.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from sublangs: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from videostore_indices database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }
    let query = "SELECT person_name FROM people 
                        INNER JOIN directors 
                        ON directors.director_id = people.person_id 
                        WHERE directors.video_id = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&video_id]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                match row.get::<usize, String>(0) {
                                    Ok(val) => v.director.push(val.clone()),
                                    Err(error) => {
                                        log::error!("Failed to read directors for video: {}", error);
                                        continue;
                                    }
                                }
                            }
                            Ok(None) => {
                                //log::warn!("No data read from indices.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from directors: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from directors database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }
    let query = "SELECT person_name FROM people 
                        INNER JOIN actors 
                        ON actors.actor_id = people.person_id 
                        WHERE actors.video_id = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&video_id]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                match row.get::<usize, String>(0) {
                                    Ok(val) => v.actors.push(val),
                                    Err(error) => {
                                        log::error!("Failed to read actors for video: {}", error);
                                        continue;
                                    }
                                }
                            }
                            Ok(None) => {
                                //log::warn!("No data read from indices.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from actors: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from actors database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }

    v
}

pub fn video(
    connection: &mut rusqlite::Connection, 
    filepath: &str,
    known_files: &mut std::collections::BTreeMap<PathBuf, crate::sql::FileMetadata>,
) -> VideoMetadata {
    let mut v = VideoMetadata {..Default::default()};
    v.path = filepath.to_string();
    let filedata = file(connection, filepath);
    let video_id = filedata.metadata_id;
    let query = "SELECT name, title, released, poster, duration, width, height, framerate, description, thumb FROM video_metadata WHERE video_id = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&video_id]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                match row.get(0) {
                                    Ok(val) => v.name = val,
                                    Err(error) => {
                                        log::error!("Failed to read id for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(1) {
                                    Ok(val) => v.title = val,
                                    Err(error) => {
                                        log::error!("Failed to read video_id for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(2) {
                                    Ok(val) => v.date = val,
                                    Err(error) => {
                                        log::error!("Failed to read screenshot_id for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(3) {
                                    Ok(val) => v.poster = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(4) {
                                    Ok(val) => v.duration = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(5) {
                                    Ok(val) => v.width = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(6) {
                                    Ok(val) => v.height = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(7) {
                                    Ok(val) => v.framerate = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(8) {
                                    Ok(val) => v.description = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(9) {
                                    Ok(val) => v.thumb = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                            },
                            Ok(None) => {
                                //log::warn!("No data read from indices.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from indices: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from videostore_indices database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }
    let query = "SELECT subpath FROM subtitles WHERE video_id = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&video_id]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                match row.get::<usize, String>(0) {
                                    Ok(val) => {
                                        let s: String = val;
                                        v.subtitles.push(s);
                                    },
                                    Err(error) => {
                                        log::error!("Failed to read id for video: {}", error);
                                        continue;
                                    }
                                }
                            }
                            Ok(None) => {
                                //log::warn!("No data read from indices.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from indices: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from subtitles database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }
    let query = "SELECT audiolang FROM audiolangs WHERE video_id = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&video_id]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                match row.get::<usize, String>(0) {
                                    Ok(val) => v.audiolangs.push(val.clone()),
                                    Err(error) => {
                                        log::error!("Failed to audiolangs for video: {}", error);
                                        continue;
                                    }
                                }
                            }
                            Ok(None) => {
                                //log::warn!("No data read from indices.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from audiolangs: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from audiolangs database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }
    let query = "SELECT sublang FROM sublangs WHERE video_id = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&video_id]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                match row.get::<usize, String>(0) {
                                    Ok(val) => v.sublangs.push(val.clone()),
                                    Err(error) => {
                                        log::error!("Failed to read sublangs for video: {}", error);
                                        continue;
                                    }
                                }
                            }
                            Ok(None) => {
                                //log::warn!("No data read from sublangs.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from sublangs: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from videostore_indices database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }
    let query = "SELECT title, start, end FROM chapters WHERE video_id = ?1 ORDER BY start, end ASC";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&video_id]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                let mut c = Chapter {..Default::default()};
                                match row.get::<usize, String>(0) {
                                    Ok(val) => c.title = val.clone(),
                                    Err(error) => {
                                        log::error!("Failed to read chapter title for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get::<usize, f32>(1) {
                                    Ok(val) => c.start = val,
                                    Err(error) => {
                                        log::error!("Failed to read chapter start for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get::<usize, f32>(2) {
                                    Ok(val) => c.end = val,
                                    Err(error) => {
                                        log::error!("Failed to read chapter end for video: {}", error);
                                        continue;
                                    }
                                }
                                v.chapters.push(c);
                            }
                            Ok(None) => {
                                //log::warn!("No data read from sublangs.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from sublangs: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from videostore_indices database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }
    let query = "SELECT person_name FROM people 
                        INNER JOIN directors 
                        ON directors.director_id = people.person_id 
                        WHERE directors.video_id = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&video_id]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                match row.get::<usize, String>(0) {
                                    Ok(val) => v.director.push(val.clone()),
                                    Err(error) => {
                                        log::error!("Failed to read directors for video: {}", error);
                                        continue;
                                    }
                                }
                            }
                            Ok(None) => {
                                //log::warn!("No data read from indices.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from directors: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from directors database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }
    let query = "SELECT person_name FROM people 
                        INNER JOIN actors 
                        ON actors.actor_id = people.person_id 
                        WHERE actors.video_id = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&video_id]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                match row.get::<usize, String>(0) {
                                    Ok(val) => v.actors.push(val),
                                    Err(error) => {
                                        log::error!("Failed to read actors for video: {}", error);
                                        continue;
                                    }
                                }
                            }
                            Ok(None) => {
                                //log::warn!("No data read from indices.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from actors: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from actors database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }
    if !known_files.contains_key(&PathBuf::from(&v.path)) {
        known_files.insert(PathBuf::from(&v.path), filedata.clone());
    }

    v
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct AudioMetadata {
    pub name: String,
    pub title: String,
    pub date: NaiveDate,
    pub path: String,
    pub poster: String,
    pub thumb: String,
    pub duration: u32,
    pub bitrate: f32,
    pub album: String,
    pub artist: Vec<String>,
    pub albumartist: Vec<String>,
    pub chapters: Vec<Chapter>,
    pub lyrics: Vec<String>,
}

impl Default for AudioMetadata {
    fn default() -> AudioMetadata {
        AudioMetadata {
            name: String::new(),
            title: String::new(),
            date: NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
            path: String::new(),
            poster: String::new(),
            thumb: String::new(),
            duration: 0,
            bitrate: 0.0,
            album: String::new(),
            artist: Vec::new(),
            albumartist: Vec::new(),
            chapters: Vec::new(),
            lyrics: Vec::new(),
        }
    }
}

pub fn insert_audio(
    connection: &mut rusqlite::Connection, 
    metadata: &mut AudioMetadata,
    statdata: &std::fs::Metadata,
    known_files: &mut std::collections::BTreeMap<PathBuf, crate::sql::FileMetadata>,
) {
    match connection.execute(
        "INSERT INTO audio_metadata (name, title, released, poster, thumb, duration, bitrate) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![&metadata.name, &metadata.title, &metadata.date, &metadata.poster, &metadata.thumb, &metadata.duration, &metadata.bitrate],
    ) {
        Ok(_retval) => {}, //log::warn!("Inserted {} video with ID {} and location {} into candidates.", video.id, video.index, candidate_id),
        Err(error) => {
            log::error!("Failed to insert audio into  database: {}", error);
            return;
        }
    }
    let mut audio_id = 0;
    let query = "SELECT last_insert_rowid()";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![]) {
                Ok(mut rows) => {
                    while let Ok(Some(row)) = rows.next() {
                        let s_opt = row.get(0);
                        if s_opt.is_ok() {
                            audio_id = s_opt.unwrap();
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from audio_metadata database: {}", err);
                }
            }
        },
        Err(error) => {
            log::error!("Failed to get audio_id for from database: {}", error);
            return;
        }
    }

    insert_file(connection, &metadata.path, statdata, 3, audio_id, known_files);

    for i in 0..metadata.artist.len() {
        let albumartist_id = insert_artist(connection, metadata.artist[i].clone());
        if albumartist_id == -1 {
            continue;
        }
        match connection.execute(
            "INSERT INTO actors (actor_id, video_id) VALUES (?1, ?2)",
            params![&albumartist_id, &audio_id],
        ) {
            Ok(_retval) => {}, //log::warn!("Inserted {} video with ID {} and location {} into candidates.", video.id, video.index, candidate_id),
            Err(error) => {
                log::error!("Failed to insert actor into  database: {}", error);
                return;
            }
        }
    }

    let _album_id = insert_album(connection, metadata.album.clone());
    for i in 0..metadata.albumartist.len() {
        let albumartist_id = insert_artist(connection, metadata.albumartist[i].clone());
        if albumartist_id == -1 {
            continue;
        }
        match connection.execute(
            "INSERT INTO actors (actor_id, video_id) VALUES (?1, ?2)",
            params![&albumartist_id, &audio_id],
        ) {
            Ok(_retval) => {}, //log::warn!("Inserted {} video with ID {} and location {} into candidates.", video.id, video.index, candidate_id),
            Err(error) => {
                log::error!("Failed to insert actor into  database: {}", error);
                return;
            }
        }
    }
    for i in 0..metadata.chapters.len() {
        match connection.execute(
            "INSERT INTO indexes (audio_id, title, start, end) VALUES (?1, ?2, ?3, ?4)",
            params![&audio_id, &metadata.chapters[i].title, &metadata.chapters[i].start, &metadata.chapters[i].end],
        ) {
            Ok(_retval) => {}, //log::warn!("Inserted {} video with ID {} and location {} into candidates.", video.id, video.index, candidate_id),
            Err(error) => {
                log::error!("Failed to insert video into  database: {}", error);
                return;
            }
        }
    }
    for i in 0..metadata.lyrics.len() {
        match connection.execute(
            "INSERT INTO lyrics (audio_id, lyricsfile) VALUES (?1, ?2)",
            params![&audio_id, &metadata.lyrics[i]],
        ) {
            Ok(_retval) => {}, //log::warn!("Inserted {} video with ID {} and location {} into candidates.", video.id, video.index, candidate_id),
            Err(error) => {
                log::error!("Failed to insert video into  database: {}", error);
                return;
            }
        }
    }
}


fn insert_album(connection: &mut rusqlite::Connection, name: String) -> i32 {
    let mut album_id= -1;
    let query = "SELECT album_id FROM albums WHERE album_name = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&name]) {
                Ok(mut rows) => {
                    while let Ok(Some(row)) = rows.next() {
                        let s_opt = row.get(1);
                        if s_opt.is_ok() {
                            album_id = s_opt.unwrap();
                            return album_id;
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from people database: {}", err);
                }
            }
        },
        Err(error) => {
            log::error!("Failed to get album_id for {} from database: {}", name, error);
            return album_id;
        }
    }
    if album_id == -1 {
        match connection.execute(
            "INSERT INTO albums (album_name) VALUES (?1)",
            params![&name],
        ) {
            Ok(_retval) => {}, //log::warn!("Inserted {} video with ID {} and location {} into candidates.", video.id, video.index, candidate_id),
            Err(error) => {
                log::error!("Failed to insert video into  database: {}", error);
                return album_id;
            }
        }

        let query = "SELECT last_insert_rowid()";
        match connection.prepare(query) {
            Ok(mut statement) => {
                match statement.query(params![]) {
                    Ok(mut rows) => {
                        while let Ok(Some(row)) = rows.next() {
                            let s_opt = row.get(0);
                            if s_opt.is_ok() {
                                album_id = s_opt.unwrap();
                            }
                        }
                    },
                    Err(err) => {
                        log::error!("could not read last generated id from database: {}", err);
                    }
                }
            },
            Err(error) => {
                log::error!("Failed to get album_id for from database: {}", error);
                return album_id;
            }
        }
    }
    album_id
}

fn insert_artist(connection: &mut rusqlite::Connection, name: String) -> i32 {
    let mut artist_id= -1;
    let query = "SELECT artist_id FROM artists WHERE artist_name = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&name]) {
                Ok(mut rows) => {
                    while let Ok(Some(row)) = rows.next() {
                        let s_opt = row.get(1);
                        if s_opt.is_ok() {
                            artist_id = s_opt.unwrap();
                            return artist_id;
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from people database: {}", err);
                }
            }
        },
        Err(error) => {
            log::error!("Failed to get artist_id for {} from database: {}", name, error);
            return artist_id;
        }
    }
    if artist_id == -1 {
        match connection.execute(
            "INSERT INTO artists (artist_name) VALUES (?1)",
            params![&name],
        ) {
            Ok(_retval) => {}, //log::warn!("Inserted {} video with ID {} and location {} into candidates.", video.id, video.index, candidate_id),
            Err(error) => {
                log::error!("Failed to insert artist into  database: {}", error);
                return artist_id;
            }
        }

        let query = "SELECT last_insert_rowid()";
        match connection.prepare(query) {
            Ok(mut statement) => {
                match statement.query(params![]) {
                    Ok(mut rows) => {
                        while let Ok(Some(row)) = rows.next() {
                            let s_opt = row.get(0);
                            if s_opt.is_ok() {
                                artist_id = s_opt.unwrap();
                            }
                        }
                    },
                    Err(err) => {
                        log::error!("could not read last generated id from database: {}", err);
                    }
                }
            },
            Err(error) => {
                log::error!("Failed to get albumartist_id for from database: {}", error);
                return artist_id;
            }
        }
    }
    artist_id
}

pub fn delete_audio(
    connection: &mut rusqlite::Connection, 
    metadata: &mut AudioMetadata,
    known_files: &mut std::collections::BTreeMap<PathBuf, crate::sql::FileMetadata>,
) {
    // Get the index.
    //let index = self.ids[id];
    let mut audio_id: u32 = 0;
    let query = "SELECT metadata_id FROM file_metadata WHERE filepath = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&metadata.path]) {
                Ok(mut rows) => {
                    while let Ok(Some(row)) = rows.next() {
                        let s_opt = row.get(1);
                        if s_opt.is_ok() {
                            audio_id = s_opt.unwrap();
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from file_metadata database: {}", err);
                }
            }
        },
        Err(error) => {
            log::error!("Failed to get video_id for {} from database: {}", metadata.path, error);
            return;
        }
    }
    // clear the entry in the candidates list without deleting it
    let ret = connection.execute(
        "DELETE FROM audio_metadata WHERE audio_id = ?1",
        params![&audio_id],
    );
    if ret.is_err() {
        log::error!("Failed to delete candidate {}!", audio_id);
        return;
    }    
    // clear the entry in the candidates list without deleting it
    let ret = connection.execute(
        "DELETE FROM artists WHERE audio_id = ?1",
        params![&audio_id],
    );
    if ret.is_err() {
        log::error!("Failed to delete subtitles {}!", audio_id);
        return;
    }    
    let ret = connection.execute(
        "DELETE FROM artist_audio_map WHERE audio_id = ?1",
        params![&audio_id],
    );
    if ret.is_err() {
        log::error!("Failed to delete artist_audio_map {}!", audio_id);
        return;
    }    
    // clear the entry in the candidates list without deleting it
    let ret = connection.execute(
        "DELETE FROM albums WHERE audio_id = ?1",
        params![&audio_id],
    );
    if ret.is_err() {
        log::error!("Failed to delete albums {}!", audio_id);
        return;
    }    
    let ret = connection.execute(
        "DELETE FROM album_audio_map WHERE audio_id = ?1",
        params![&audio_id],
    );
    if ret.is_err() {
        log::error!("Failed to delete albums {}!", audio_id);
        return;
    }    
    // clear the entry in the candidates list without deleting it
    let ret = connection.execute(
        "DELETE FROM albumartist_audio_map WHERE audio_id = ?1",
        params![&audio_id],
    );
    if ret.is_err() {
        log::error!("Failed to delete albumartist_audio_map {}!", audio_id);
        return;
    }    
    // clear the entry in the candidates list without deleting it
    let ret = connection.execute(
        "DELETE FROM file_metadata WHERE filepath = ?1",
        params![&metadata.path],
    );
    if ret.is_err() {
        log::error!("Failed to delete candidate {}!", metadata.path);
        return;
    }
    known_files.remove(&PathBuf::from(&metadata.path));
}

pub fn update_audio(
    connection: &mut rusqlite::Connection, 
    metadata: &mut crate::sql::AudioMetadata,
    statdata: &std::fs::Metadata,
    known_files: &mut std::collections::BTreeMap<PathBuf, crate::sql::FileMetadata>,
) {
    delete_audio(connection, metadata, known_files);
    insert_audio(connection, metadata, statdata, known_files);
}

pub fn audio_by_id(
    connection: &mut rusqlite::Connection, 
    filepath: &str,
    audio_id: i64,
) -> AudioMetadata {
    let mut v = AudioMetadata {..Default::default()};
    v.path = filepath.to_string();
    // fill v from all tables
    let query = "SELECT name, title, released, poster, thumb, duration FROM audio_metadata WHERE audio_id = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&audio_id]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                match row.get(0) {
                                    Ok(val) => v.name = val,
                                    Err(error) => {
                                        log::error!("Failed to read name for audio: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(1) {
                                    Ok(val) => v.title = val,
                                    Err(error) => {
                                        log::error!("Failed to read title for audio: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(2) {
                                    Ok(val) => v.date = val,
                                    Err(error) => {
                                        log::error!("Failed to read date for audio: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(3) {
                                    Ok(val) => v.poster = val,
                                    Err(error) => {
                                        log::error!("Failed to read poster for audio: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(4) {
                                    Ok(val) => v.thumb = val,
                                    Err(error) => {
                                        log::error!("Failed to read bitrate for audio: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(5) {
                                    Ok(val) => v.duration = val,
                                    Err(error) => {
                                        log::error!("Failed to read duration for audio: {}", error);
                                        continue;
                                    }
                                }
                            },
                            Ok(None) => {
                                //log::warn!("No data read from indices.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from indices: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from videostore_indices database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }
    let query = "SELECT artist_name FROM artists 
                        INNER JOIN artist_audio_map 
                        ON artist_audio_map.artist_id = artists.artist_id 
                        WHERE artist_audio_map.audio_id = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&audio_id]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                match row.get::<usize, String>(0) {
                                    Ok(val) => {
                                        let s: String = val.to_string();
                                        v.artist.push(s);
                                    },
                                    Err(error) => {
                                        log::error!("Failed to read artists for video: {}", error);
                                        continue;
                                    }
                                }
                            }
                            Ok(None) => {
                                //log::warn!("No data read from indices.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from artists: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from artists database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }
    let query = "SELECT album_name FROM albums 
                        INNER JOIN album_audio_map 
                        ON album_audio_map.album_id = albums.album_id 
                        WHERE album_audio_map.audio_id = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&audio_id]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                match row.get::<usize, String>(0) {
                                    Ok(val) => v.album = val.clone(),
                                    Err(error) => {
                                        log::error!("Failed to read albums for video: {}", error);
                                        continue;
                                    }
                                }
                            }
                            Ok(None) => {
                                //log::warn!("No data read from indices.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from albums: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from albums database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }
    let query = "SELECT artist_name FROM artists 
                        INNER JOIN albumartist_audio_map 
                        ON albumartist_audio_map.albumartist_id = artists.artist_id 
                        WHERE albumartist_audio_map.audio_id = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&audio_id]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                match row.get::<usize, String>(0) {
                                    Ok(val) => v.albumartist.push(val.clone()),
                                    Err(error) => {
                                        log::error!("Failed to read album_artists for audio: {}", error);
                                        continue;
                                    }
                                }
                            }
                            Ok(None) => {
                                //log::warn!("No data read from indices.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from album_artists: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from album_artists database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }
    let query = "SELECT title, start, end FROM indexes WHERE audio_id = ?1 ORDER BY start, end ASC";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&audio_id]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                let mut c = Chapter {..Default::default()};
                                match row.get::<usize, String>(0) {
                                    Ok(val) => c.title = val.clone(),
                                    Err(error) => {
                                        log::error!("Failed to read chapter title for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get::<usize, f32>(1) {
                                    Ok(val) => c.start = val,
                                    Err(error) => {
                                        log::error!("Failed to read chapter start for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get::<usize, f32>(2) {
                                    Ok(val) => c.end = val,
                                    Err(error) => {
                                        log::error!("Failed to read chapter end for video: {}", error);
                                        continue;
                                    }
                                }
                                v.chapters.push(c);
                            }
                            Ok(None) => {
                                //log::warn!("No data read from sublangs.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from sublangs: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from videostore_indices database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }
    let query = "SELECT lyricsfile FROM lyrics WHERE audio_id = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&audio_id]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                match row.get::<usize, String>(0) {
                                    Ok(val) => {
                                        let s = val.clone();
                                        v.lyrics.push(s);
                                    }
                                    Err(error) => {
                                        log::error!("Failed to read chapter title for video: {}", error);
                                        continue;
                                    }
                                }
                            }
                            Ok(None) => {
                                //log::warn!("No data read from sublangs.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from sublangs: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from videostore_indices database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }
    v
}

pub fn audio(
    connection: &mut rusqlite::Connection, 
    filepath: &str,
    known_files: &mut std::collections::BTreeMap<PathBuf, crate::sql::FileMetadata>,
) -> AudioMetadata {
    let mut v = AudioMetadata {..Default::default()};
    // fill v from all tables
    v.path = filepath.to_string();
    let filedata = file(connection, filepath);
    let audio_id = filedata.metadata_id;
    let query = "SELECT name, title, released, poster, thumb, duration FROM audio_metadata WHERE audio_id = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&audio_id]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                match row.get(0) {
                                    Ok(val) => v.name = val,
                                    Err(error) => {
                                        log::error!("Failed to read name for audio: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(1) {
                                    Ok(val) => v.title = val,
                                    Err(error) => {
                                        log::error!("Failed to read title for audio: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(2) {
                                    Ok(val) => v.date = val,
                                    Err(error) => {
                                        log::error!("Failed to read date for audio: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(3) {
                                    Ok(val) => v.poster = val,
                                    Err(error) => {
                                        log::error!("Failed to read poster for audio: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(4) {
                                    Ok(val) => v.thumb = val,
                                    Err(error) => {
                                        log::error!("Failed to read bitrate for audio: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(5) {
                                    Ok(val) => v.duration = val,
                                    Err(error) => {
                                        log::error!("Failed to read duration for audio: {}", error);
                                        continue;
                                    }
                                }
                            },
                            Ok(None) => {
                                //log::warn!("No data read from indices.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from indices: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from videostore_indices database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }
    let query = "SELECT artist_name FROM artists 
                        INNER JOIN artist_audio_map 
                        ON artist_audio_map.artist_id = artists.artist_id 
                        WHERE artist_audio_map.audio_id = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&audio_id]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                match row.get::<usize, String>(0) {
                                    Ok(val) => {
                                        let s: String = val.to_string();
                                        v.artist.push(s);
                                    },
                                    Err(error) => {
                                        log::error!("Failed to read artists for video: {}", error);
                                        continue;
                                    }
                                }
                            }
                            Ok(None) => {
                                //log::warn!("No data read from indices.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from artists: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from artists database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }
    let query = "SELECT album_name FROM albums 
                        INNER JOIN album_audio_map 
                        ON album_audio_map.album_id = albums.album_id 
                        WHERE album_audio_map.audio_id = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&audio_id]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                match row.get::<usize, String>(0) {
                                    Ok(val) => v.album = val.clone(),
                                    Err(error) => {
                                        log::error!("Failed to read albums for video: {}", error);
                                        continue;
                                    }
                                }
                            }
                            Ok(None) => {
                                //log::warn!("No data read from indices.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from albums: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from albums database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }
    let query = "SELECT artist_name FROM artists 
                        INNER JOIN albumartist_audio_map 
                        ON albumartist_audio_map.albumartist_id = artists.artist_id 
                        WHERE albumartist_audio_map.audio_id = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&audio_id]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                match row.get::<usize, String>(0) {
                                    Ok(val) => v.albumartist.push(val.clone()),
                                    Err(error) => {
                                        log::error!("Failed to read album_artists for audio: {}", error);
                                        continue;
                                    }
                                }
                            }
                            Ok(None) => {
                                //log::warn!("No data read from indices.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from album_artists: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from album_artists database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }
    let query = "SELECT title, start, end FROM indexes WHERE audio_id = ?1 ORDER BY start, end ASC";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&audio_id]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                let mut c = Chapter {..Default::default()};
                                match row.get::<usize, String>(0) {
                                    Ok(val) => c.title = val.clone(),
                                    Err(error) => {
                                        log::error!("Failed to read chapter title for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get::<usize, f32>(1) {
                                    Ok(val) => c.start = val,
                                    Err(error) => {
                                        log::error!("Failed to read chapter start for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get::<usize, f32>(2) {
                                    Ok(val) => c.end = val,
                                    Err(error) => {
                                        log::error!("Failed to read chapter end for video: {}", error);
                                        continue;
                                    }
                                }
                                v.chapters.push(c);
                            }
                            Ok(None) => {
                                //log::warn!("No data read from sublangs.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from sublangs: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from videostore_indices database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }
    let query = "SELECT lyricsfile FROM lyrics WHERE audio_id = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&audio_id]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                match row.get::<usize, String>(0) {
                                    Ok(val) => {
                                        let s = val.clone();
                                        v.lyrics.push(s);
                                    }
                                    Err(error) => {
                                        log::error!("Failed to read chapter title for video: {}", error);
                                        continue;
                                    }
                                }
                            }
                            Ok(None) => {
                                //log::warn!("No data read from sublangs.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from sublangs: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from videostore_indices database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }
    if !known_files.contains_key(&PathBuf::from(&v.path)) {
        known_files.insert(PathBuf::from(&v.path), filedata.clone());
    }
    v
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct ImageMetadata {
    pub name: String,
    pub title: String,
    pub date: NaiveDate,
    pub path: String,
    pub resized: String,
    pub thumb: String,
    pub width: u32,
    pub height: u32,
    pub photographer: String,
    pub lense_model: String,
    pub focal_length: String,
    pub exposure_time: String,
    pub fnumber: String,
    pub gps_string: String,
    pub gps_latitude: f32,
    pub gps_longitude: f32,
    pub gps_altitude: f32,
}

impl Default for ImageMetadata {
    fn default() -> ImageMetadata {
        ImageMetadata {
            name: String::new(),
            title: String::new(),
            date: NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
            path: String::new(),
            resized: String::new(),
            thumb: String::new(),
            width: 0,
            height: 0,
            photographer: String::new(),
            lense_model: String::new(),
            focal_length: String::new(),
            exposure_time: String::new(),
            fnumber: String::new(),
            gps_string: String::new(),
            gps_latitude: 0.0,
            gps_longitude: 0.0,
            gps_altitude: 0.0,
        }
    }
}

pub fn insert_image(
    connection: &mut rusqlite::Connection, 
    metadata: &mut ImageMetadata,
    statdata: &std::fs::Metadata,
    known_files: &mut std::collections::BTreeMap<PathBuf, crate::sql::FileMetadata>,
) {
    match connection.execute(
        "INSERT INTO image_metadata (name, path, created, resized, thumb, width, height, photographer, LenseModel, Focallength, Exposuretime, FNumber, gpsstring, gpslatitude, gpslongitude, gpsaltitude) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
        params![&metadata.name, &metadata.path, &metadata.date, &metadata.resized, &metadata.thumb, &metadata.width, &metadata.height, &metadata.photographer, &metadata.lense_model, &metadata.focal_length, &metadata.exposure_time, &metadata.fnumber, &metadata.gps_string, &metadata.gps_latitude, &metadata.gps_longitude, &metadata.gps_altitude],
    ) {
        Ok(_retval) => {}, //log::warn!("Inserted {} image with ID {} and location {} into candidates.", video.id, video.index, candidate_id),
        Err(error) => {
            log::error!("Failed to insert image into  database: {}", error);
            return;
        }
    }
    let mut image_id = 0;
    let query = "SELECT last_insert_rowid()";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![]) {
                Ok(mut rows) => {
                    while let Ok(Some(row)) = rows.next() {
                        let s_opt = row.get(0);
                        if s_opt.is_ok() {
                            image_id = s_opt.unwrap();
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from audio_metadata database: {}", err);
                }
            }
        },
        Err(error) => {
            log::error!("Failed to get audio_id for from database: {}", error);
            return;
        }
    }

    insert_file(connection, &metadata.path, statdata, 1, image_id, known_files);

}

pub fn delete_image(
    connection: &mut rusqlite::Connection, 
    metadata: &mut ImageMetadata,
    known_files: &mut std::collections::BTreeMap<PathBuf, crate::sql::FileMetadata>,
) {
    // Get the index.
    //let index = self.ids[id];
    let mut image_id: u32 = 0;
    let query = "SELECT metadata_id FROM file_metadata WHERE filepath = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&metadata.path]) {
                Ok(mut rows) => {
                    while let Ok(Some(row)) = rows.next() {
                        let s_opt = row.get(1);
                        if s_opt.is_ok() {
                            image_id = s_opt.unwrap();
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from file_metadata database: {}", err);
                }
            }
        },
        Err(error) => {
            log::error!("Failed to get video_id for {} from database: {}", metadata.path, error);
            return;
        }
    }
    // clear the entry in the candidates list without deleting it
    let ret = connection.execute(
        "DELETE FROM image_metadata WHERE image_id = ?1",
        params![&image_id],
    );
    if ret.is_err() {
        log::error!("Failed to delete candidate {}!", image_id);
        return;
    }    
    known_files.remove(&PathBuf::from(&metadata.path));
}

pub fn update_image(
    connection: &mut rusqlite::Connection, 
    metadata: &mut crate::sql::ImageMetadata,
    statdata: &std::fs::Metadata,
    known_files: &mut std::collections::BTreeMap<PathBuf, crate::sql::FileMetadata>,
) {
    delete_image(connection, metadata, known_files);
    insert_image(connection, metadata, statdata, known_files);
}

pub fn image_by_id(
    connection: &mut rusqlite::Connection, 
    filepath: &str,
    image_id: i64,
) -> ImageMetadata {
    let mut v = ImageMetadata {..Default::default()};
    v.path = filepath.to_string();
    // fill v from all tables
    let query = "SELECT name, path, created, resized, thumb, width, height, Photographer, LenseModel, Focallength, Exposuretime, FNumber, GPSLatitude, GPSLongitude, GPSAltitude FROM image_metadata WHERE image_id = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&image_id]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                match row.get(0) {
                                    Ok(val) => v.name = val,
                                    Err(error) => {
                                        log::error!("Failed to read name for image: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(1) {
                                    Ok(val) => v.path = val,
                                    Err(error) => {
                                        log::error!("Failed to read title for image: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(2) {
                                    Ok(val) => v.date = val,
                                    Err(error) => {
                                        log::error!("Failed to read date for image: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(3) {
                                    Ok(val) => v.resized = val,
                                    Err(error) => {
                                        log::error!("Failed to read resized for image: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(4) {
                                    Ok(val) => v.thumb = val,
                                    Err(error) => {
                                        log::error!("Failed to read thumb for image: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(5) {
                                    Ok(val) => v.width = val,
                                    Err(error) => {
                                        log::error!("Failed to read width for image: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(6) {
                                    Ok(val) => v.height = val,
                                    Err(error) => {
                                        log::error!("Failed to read height for image: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(7) {
                                    Ok(val) => v.photographer = val,
                                    Err(error) => {
                                        log::error!("Failed to read photographer for image: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(8) {
                                    Ok(val) => v.lense_model = val,
                                    Err(error) => {
                                        log::error!("Failed to read lense_model for image: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(9) {
                                    Ok(val) => v.focal_length = val,
                                    Err(error) => {
                                        log::error!("Failed to read focal_length for image: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(10) {
                                    Ok(val) => v.exposure_time = val,
                                    Err(error) => {
                                        log::error!("Failed to read exposure_time for image: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(11) {
                                    Ok(val) => v.fnumber = val,
                                    Err(error) => {
                                        log::error!("Failed to read fnumber for image: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(12) {
                                    Ok(val) => v.gps_latitude = val,
                                    Err(error) => {
                                        log::error!("Failed to read gps_latitude for image: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(13) {
                                    Ok(val) => v.gps_longitude = val,
                                    Err(error) => {
                                        log::error!("Failed to read gps_longitude for image: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(14) {
                                    Ok(val) => v.gps_altitude = val,
                                    Err(error) => {
                                        log::error!("Failed to read gps_altitude for image: {}", error);
                                        continue;
                                    }
                                }
                            },
                            Ok(None) => {
                                //log::warn!("No data read from indices.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from indices: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from videostore_indices database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }
    v
}

pub fn image(
    connection: &mut rusqlite::Connection, 
    filepath: &str,
    known_files: &mut std::collections::BTreeMap<PathBuf, crate::sql::FileMetadata>,
) -> ImageMetadata {
    let mut v = ImageMetadata {..Default::default()};
    // fill v from all tables
    v.path = filepath.to_string();
    let filedata = file(connection, filepath);
    let image_id = filedata.metadata_id;
    let query = "SELECT name, path, created, resized, thumb, width, height, Photographer, LenseModel, Focallength, Exposuretime, FNumber, GPSLatitude, GPSLongitude, GPSAltitude FROM image_metadata WHERE image_id = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&image_id]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                match row.get(0) {
                                    Ok(val) => v.name = val,
                                    Err(error) => {
                                        log::error!("Failed to read name for image: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(1) {
                                    Ok(val) => v.path = val,
                                    Err(error) => {
                                        log::error!("Failed to read title for image: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(2) {
                                    Ok(val) => v.date = val,
                                    Err(error) => {
                                        log::error!("Failed to read date for image: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(3) {
                                    Ok(val) => v.resized = val,
                                    Err(error) => {
                                        log::error!("Failed to read resized for image: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(4) {
                                    Ok(val) => v.thumb = val,
                                    Err(error) => {
                                        log::error!("Failed to read thumb for image: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(5) {
                                    Ok(val) => v.width = val,
                                    Err(error) => {
                                        log::error!("Failed to read width for image: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(6) {
                                    Ok(val) => v.height = val,
                                    Err(error) => {
                                        log::error!("Failed to read height for image: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(7) {
                                    Ok(val) => v.photographer = val,
                                    Err(error) => {
                                        log::error!("Failed to read photographer for image: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(8) {
                                    Ok(val) => v.lense_model = val,
                                    Err(error) => {
                                        log::error!("Failed to read lense_model for image: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(9) {
                                    Ok(val) => v.focal_length = val,
                                    Err(error) => {
                                        log::error!("Failed to read focal_length for image: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(10) {
                                    Ok(val) => v.exposure_time = val,
                                    Err(error) => {
                                        log::error!("Failed to read exposure_time for image: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(11) {
                                    Ok(val) => v.fnumber = val,
                                    Err(error) => {
                                        log::error!("Failed to read fnumber for image: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(12) {
                                    Ok(val) => v.gps_latitude = val,
                                    Err(error) => {
                                        log::error!("Failed to read gps_latitude for image: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(13) {
                                    Ok(val) => v.gps_longitude = val,
                                    Err(error) => {
                                        log::error!("Failed to read gps_longitude for image: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(14) {
                                    Ok(val) => v.gps_altitude = val,
                                    Err(error) => {
                                        log::error!("Failed to read gps_altitude for image: {}", error);
                                        continue;
                                    }
                                }
                            },
                            Ok(None) => {
                                //log::warn!("No data read from indices.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from indices: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from videostore_indices database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }
    if !known_files.contains_key(&PathBuf::from(&v.path)) {
        known_files.insert(PathBuf::from(&v.path), filedata.clone());
    }
    v
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct FileMetadata {
    pub filepath: PathBuf,
    pub creation_time: u64,
    pub modification_time: u64,
    pub file_type: i32,
    pub metadata_id: i64,
}

impl Default for FileMetadata {
    fn default() -> FileMetadata {
        FileMetadata {
            filepath: PathBuf::new(),
            creation_time: 0,
            modification_time: 0,
            file_type: 0,
            metadata_id: -1,
        }
    }
}

pub fn file_by_id(
    connection: &mut rusqlite::Connection, 
    metadata_id: i64,
) -> FileMetadata {
    let mut v = FileMetadata {..Default::default()};
    let query = "SELECT * FROM file_metadata WHERE metadata_id = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&metadata_id]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                match row.get(0) {
                                    Ok(val) => {
                                        let st: String = val;
                                        v.filepath = PathBuf::from(&st);
                                    },
                                    Err(error) => {
                                        log::error!("Failed to read id for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(1) {
                                    Ok(val) => v.creation_time = val,
                                    Err(error) => {
                                        log::error!("Failed to read video_id for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(2) {
                                    Ok(val) => v.modification_time = val,
                                    Err(error) => {
                                        log::error!("Failed to read screenshot_id for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(3) {
                                    Ok(val) => v.file_type = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(4) {
                                    Ok(val) => v.metadata_id = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                            },
                            Ok(None) => {
                                //log::warn!("No data read from indices.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from indices: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from videostore_indices database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }

    v
}

pub fn file(
    connection: &mut rusqlite::Connection, 
    filepath: &str,
) -> FileMetadata {
    let mut v = FileMetadata {..Default::default()};
    let query = "SELECT * FROM file_metadata WHERE filepath = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![filepath]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                match row.get(0) {
                                    Ok(val) => {
                                        let st: String = val;
                                        v.filepath = PathBuf::from(&st);
                                    },
                                    Err(error) => {
                                        log::error!("Failed to read id for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(1) {
                                    Ok(val) => v.creation_time = val,
                                    Err(error) => {
                                        log::error!("Failed to read video_id for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(2) {
                                    Ok(val) => v.modification_time = val,
                                    Err(error) => {
                                        log::error!("Failed to read screenshot_id for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(3) {
                                    Ok(val) => v.file_type = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(4) {
                                    Ok(val) => v.metadata_id = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                            },
                            Ok(None) => {
                                //log::warn!("No data read from indices.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from indices: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from videostore_indices database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }

    v
}


pub fn files(connection: &mut rusqlite::Connection) -> std::collections::BTreeMap<PathBuf, FileMetadata> {
    let mut known_files = BTreeMap::new();
    let query = "SELECT * FROM file_metadata";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                let mut s = FileMetadata {..Default::default()};
                                match row.get(0) {
                                    Ok(val) => {
                                        let st: String = val;
                                        s.filepath = PathBuf::from(&st);
                                    },
                                    Err(error) => {
                                        log::error!("Failed to read id for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(1) {
                                    Ok(val) => s.creation_time = val,
                                    Err(error) => {
                                        log::error!("Failed to read video_id for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(2) {
                                    Ok(val) => s.modification_time = val,
                                    Err(error) => {
                                        log::error!("Failed to read screenshot_id for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(3) {
                                    Ok(val) => s.file_type = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(4) {
                                    Ok(val) => s.metadata_id = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                known_files.insert(s.filepath.clone(), s.clone());
                                if s.file_type == 3 {
                                    let mut thumbstring = crate::parsers::osstr_to_string(s.filepath.clone().into_os_string());
                                    thumbstring.extend(".png".to_string().chars());
                                    let thumbpath = PathBuf::from(&thumbstring);
                                    if !known_files.contains_key(&thumbpath) {
                                        known_files.insert(thumbpath, s.clone());
                                    }
                                    let mut lyricstring = crate::parsers::osstr_to_string(s.filepath.clone().into_os_string());
                                    lyricstring.extend(".lrc".to_string().chars());
                                    let lyricpath = PathBuf::from(&lyricstring);
                                    if !known_files.contains_key(&lyricpath) {
                                        known_files.insert(lyricpath, s);
                                    }
                                } else if s.file_type == 2 {
                                    let mut thumbstring = crate::parsers::osstr_to_string(s.filepath.clone().into_os_string());
                                    thumbstring.extend("_001.jpeg".to_string().chars());
                                    let thumbpath = PathBuf::from(&thumbstring);
                                    if !known_files.contains_key(&thumbpath) {
                                        known_files.insert(thumbpath, s);
                                    }
                                }
                                                        },
                            Ok(None) => {
                                //log::warn!("No data read from indices.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from indices: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from videostore_indices database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }

    known_files
}

pub fn insert_file(
    connection: &mut rusqlite::Connection, 
    path: &str,
    metadata: &std::fs::Metadata,
    file_type: i32,
    metadata_id: i64,
    known_files: &mut std::collections::BTreeMap<PathBuf, crate::sql::FileMetadata>,
) {
    let mut creation_time: u64 = 0;
    if let Ok(created) = metadata.created() {
        match created.duration_since(std::time::UNIX_EPOCH) {
            Ok(n) => creation_time = n.as_secs(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        }
    }
    let mut modification_time: u64 = 0;
    if let Ok(modified) = metadata.modified() {
        match modified.duration_since(std::time::UNIX_EPOCH) {
            Ok(n) => modification_time = n.as_secs(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        }
    }
    match connection.execute(
        "INSERT INTO file_metadata (filepath, creation_time, modification_time, 
            file_type, metadata_id) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![&path, &creation_time, &modification_time, &file_type, &metadata_id],
    ) {
        Ok(_retval) => {}, //log::warn!("Inserted {} video with ID {} and location {} into candidates.", video.id, video.index, candidate_id),
        Err(error) => {
            log::error!("Failed to insert file into  database: {}", error);
            return;
        }
    }
    let meta = FileMetadata {
        filepath: PathBuf::from(&path),
        creation_time,
        modification_time,
        file_type,
        metadata_id,
        ..Default::default()
    };
    known_files.insert(meta.filepath.clone(), meta.clone());

    if file_type == 3 {
        let mut thumbstring = path.to_string();
        thumbstring.extend(".png".to_string().chars());
        let thumbpath = PathBuf::from(&thumbstring);
        if !known_files.contains_key(&thumbpath) {
            known_files.insert(thumbpath, meta.clone());
        }
        let mut lyricstring = path.to_string();
        lyricstring.extend(".lrc".to_string().chars());
        let lyricpath = PathBuf::from(&lyricstring);
        if !known_files.contains_key(&lyricpath) {
            known_files.insert(lyricpath, meta);
        }
    } else if file_type == 2 {
        let mut thumbstring = path.to_string();
        thumbstring.extend("_001.jpeg".to_string().chars());
        let thumbpath = PathBuf::from(&thumbstring);
        if !known_files.contains_key(&thumbpath) {
            known_files.insert(thumbpath, meta);
        }
    }
}

pub fn delete_file(    
    connection: &mut rusqlite::Connection, 
    path: &str,
    known_files: &mut std::collections::BTreeMap<PathBuf, crate::sql::FileMetadata>,
) {
    let _ret = connection.execute(
        "DELETE FROM file_metadata WHERE filepath = ?1",
        params![&path],
    );
    known_files.remove(&PathBuf::from(path));
}

pub fn update_file(    
    connection: &mut rusqlite::Connection, 
    path: &str,
    metadata: &std::fs::Metadata,
    file_type: i32,
    metadata_id: i64,
    known_files: &mut std::collections::BTreeMap<PathBuf, crate::sql::FileMetadata>,
) {
    delete_file(connection, path, known_files);
    insert_file(connection, path, metadata, file_type, metadata_id, known_files);
}   


pub fn insert_search(
    connection: &mut rusqlite::Connection, 
    s: SearchData,
) {
    let fromvalue = (s.from_value / 1000000) as f32;
    let tovalue = (s.to_value / 1000000) as f32;
    match connection.execute(
        "INSERT INTO searches (search_string, from_string, from_value, to_string, to_value,
                image, video, audio, filepath, title, description, actor, director, artist, album_artist,
                duration, creation_date, modification_date, release_date, 
                lense_model, focal_length, exposure_time, fnumber,
                gps_latitude, gps_longitude, gps_altitude) VALUES 
                (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26)",
        params![&s.search_string, &s.from_string, &fromvalue, &s.to_string, &tovalue , 
                &s.image, &s.video, &s.audio, &s.filepath, &s.title, &s.description, &s.actor, &s.director, &s.artist, &s.album_artist,
                &s.duration, &s.creation_date, &s.modification_date, &s.release_date,
                &s.lense_model, &s.focal_length, &s.exposure_time, &s.fnumber,
                &s.gps_latitude, &s.gps_longitude, &s.gps_altitude],
    ) {
        Ok(_retval) => {}, //log::warn!("Inserted {} video with ID {} and location {} into candidates.", video.id, video.index, candidate_id),
        Err(error) => {
            log::error!("Failed to insert video into  database: {}", error);
            return;
        }
    }
}

pub fn delete_search(    
    connection: &mut rusqlite::Connection, 
    s: SearchData,
) {
    let _ret = connection.execute(
        "DELETE FROM searches WHERE search_id = ?1",
        params![&s.search_id],
    );
}

pub fn update_search(    
    connection: &mut rusqlite::Connection, 
    s: SearchData,
) {
    delete_search(connection, s.clone());
    insert_search(connection, s);
}   

pub fn searches(
    connection: &mut rusqlite::Connection, 
) -> Vec<SearchData> {
    let mut searches = Vec::new();
    let query = "SELECT * FROM searches";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                let mut v = SearchData {..Default::default()};
                                match row.get(0) {
                                    Ok(val) => {
                                        v.search_id = val;
                                    },
                                    Err(error) => {
                                        log::error!("Failed to read id for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(1) {
                                    Ok(val) => v.search_string = val,
                                    Err(error) => {
                                        log::error!("Failed to read video_id for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(2) {
                                    Ok(val) => v.from_string = val,
                                    Err(error) => {
                                        log::error!("Failed to read screenshot_id for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(3) {
                                    Ok(val) => {
                                        let f: f32 = val;
                                        v.from_value = (f * 1000000.0) as u32;
                                    },
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(4) {
                                    Ok(val) => v.to_string = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(5) {
                                    Ok(val) => {
                                        let f: f32 = val;
                                        v.to_value = (f * 1000000.0) as u32;
                                    },
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(6) {
                                    Ok(val) => v.image = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(7) {
                                    Ok(val) => v.video = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(8) {
                                    Ok(val) => v.audio = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(9) {
                                    Ok(val) => v.filepath = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(10) {
                                    Ok(val) => v.title = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(11) {
                                    Ok(val) => v.description = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(12) {
                                    Ok(val) => v.actor = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(13) {
                                    Ok(val) => v.director = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(14) {
                                    Ok(val) => v.artist = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(15) {
                                    Ok(val) => v.album_artist = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(16) {
                                    Ok(val) => v.duration = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(17) {
                                    Ok(val) => v.creation_date = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(18) {
                                    Ok(val) => v.modification_date = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(19) {
                                    Ok(val) => v.release_date = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(20) {
                                    Ok(val) => v.lense_model = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(21) {
                                    Ok(val) => v.focal_length = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(22) {
                                    Ok(val) => v.exposure_time = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(23) {
                                    Ok(val) => v.fnumber = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(24) {
                                    Ok(val) => v.gps_latitude = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(25) {
                                    Ok(val) => v.gps_longitude = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(26) {
                                    Ok(val) => v.gps_altitude = val,
                                    Err(error) => {
                                        log::error!("Failed to read runtime for video: {}", error);
                                        continue;
                                    }
                                }
                                searches.push(v);
                            },
                            Ok(None) => {
                                //log::warn!("No data read from indices.");
                                break;
                            },
                            Err(error) => {
                                log::error!("Failed to read a row from indices: {}", error);
                                break;
                            }
                        }
                    }
                },
                Err(err) => {
                    log::error!("could not read line from videostore_indices database: {}", err);
                }
            }
        },
        Err(err) => {
            log::error!("could not prepare SQL statement: {}", err);
        }
    }

    searches
}

pub fn connect() -> Result<rusqlite::Connection, rusqlite::Error> {
    let sqlite_file;
    let connection;
    match dirs::data_local_dir() {
        Some(pb) => {
            let mut dir = pb.join("media-browser");
            if !dir.exists() {
                let ret = std::fs::create_dir_all(dir.clone());
                if ret.is_err() {
                    log::warn!("Failed to create directory {}", dir.display());
                    dir = dirs::home_dir().unwrap();
                }
            }
            sqlite_file = dir.join("metadata.sqlite");
        },
        None => {
            let dir = dirs::home_dir().unwrap();
            sqlite_file = dir.join("metadata.sqlite");
        },
    }
    if !sqlite_file.is_file() {
        connection = Connection::open(sqlite_file)?;
        println!("{}", connection.is_autocommit());
        // file_type: 0 other file, 1 image, 2, video, 3 audio
        match connection.execute(
            "CREATE TABLE file_metadata (
                filepath TEXT NOT NULL unique PRIMARY KEY NOT NULL, 
                creation_time UNSIGNED BIG INT, 
                modification_time UNSIGNED BIG INT, 
                file_type INT,   
                metadata_id BIG INT
            )", (),
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create table file_metadata: {}", error);
                return Err(error);
            }
        }
        match connection.execute(
            "CREATE INDEX index_file_metadata_metadata_id ON file_metadata (metadata_id)", (),
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create index on file_metadata: {}", error);
                return Err(error);
            }
        }
        match connection.execute("
            CREATE TABLE video_metadata (
                video_id INTEGER,
                name  TEXT NOT NULL, 
                title  TEXT NOT NULL, 
                released UNSIGNED BIG INT NOT NULL, 
                poster  TEXT, 
                thumb   TEXT,
                subtitles BIG INT,
                duration INT,
                width INT,
                height INT,
                framerate FLOAT,
                description TEXT,
                PRIMARY KEY(video_id AUTOINCREMENT)
            )", [],
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create table indices: {}", error);
                return Err(error);
            }
        }
        match connection.execute(
            "CREATE INDEX index_video_metadata_title ON video_metadata (title)", (),
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create index on file_metadata: {}", error);
                return Err(error);
            }
        }
        match connection.execute("
            CREATE TABLE subtitles (
                subtitle_id INTEGER, 
                video_id INTEGER, 
                subpath TEXT, 
                PRIMARY KEY(subtitle_id AUTOINCREMENT)
            )", [],
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create table parameters: {}", error);
                return Err(error);
            }
        }
        match connection.execute(
            "CREATE INDEX index_subtitles_video_id ON subtitles (video_id)", (),
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create index on subtitles: {}", error);
                return Err(error);
            }
        }
        match connection.execute("
            CREATE TABLE audiolangs (
                audiolang_id INTEGER, 
                video_id INTEGER, 
                audiolang TEXT, 
                PRIMARY KEY(audiolang_id AUTOINCREMENT)
            )", [],
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create table parameters: {}", error);
                return Err(error);
            }
        }
        match connection.execute(
            "CREATE INDEX index_audiolangs_video_id ON audiolangs (video_id)", (),
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create index on audiolangs: {}", error);
                return Err(error);
            }
        }
        match connection.execute("
            CREATE TABLE sublangs (
                sublang_id INTEGER, 
                video_id INTEGER, 
                sublang TEXT, 
                PRIMARY KEY(sublang_id AUTOINCREMENT)
            )", [],
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create table parameters: {}", error);
                return Err(error);
            }
        }
        match connection.execute(
            "CREATE INDEX index_sublangs_video_id ON sublangs (video_id)", (),
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create index on sublangs: {}", error);
                return Err(error);
            }
        }
        match connection.execute("
            CREATE TABLE chapters (
                chapter_id INTEGER, 
                video_id INTEGER, 
                title TEXT, 
                start DOUBLE,
                end DOUBLE,
                PRIMARY KEY(chapter_id AUTOINCREMENT)
            )", [],
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create table parameters: {}", error);
                return Err(error);
            }
        }
        match connection.execute(
            "CREATE INDEX index_chapters_video_id ON chapters (video_id)", (),
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create index on sublangs: {}", error);
                return Err(error);
            }
        }
        match connection.execute("
        CREATE TABLE people (
            person_id INTEGER, 
            person_name TEXT, 
            PRIMARY KEY(person_id AUTOINCREMENT)
        )", [],
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create table people: {}", error);
                return Err(error);
            }
        }
        match connection.execute(
            "CREATE INDEX index_people_name ON people (person_name)", (),
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create index on directors: {}", error);
                return Err(error);
            }
        }
        match connection.execute("
            CREATE TABLE directors (
                entry_id INTEGER,
                director_id INTEGER, 
                video_id INTEGER, 
                PRIMARY KEY(entry_id AUTOINCREMENT)
            )", [],
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create table directors: {}", error);
                return Err(error);
            }
        }
        match connection.execute(
            "CREATE INDEX index_directors_video_id ON directors (video_id)", (),
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create index on directors: {}", error);
                return Err(error);
            }
        }
        match connection.execute("
            CREATE TABLE actors (
                entry_id INTEGER,
                actor_id INTEGER, 
                video_id INTEGER, 
                PRIMARY KEY(entry_id AUTOINCREMENT)
            )", [],
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create table parameters: {}", error);
                return Err(error);
            }
        }
         match connection.execute(
            "CREATE INDEX index_actors_video_id ON actors (video_id)", (),
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create index on actors: {}", error);
                return Err(error);
            }
        }

        match connection.execute("
            CREATE TABLE audio_metadata (
                audio_id INTEGER,
                name  TEXT NOT NULL, 
                title  TEXT NOT NULL, 
                released UNSIGNED BIG INT NOT NULL, 
                poster  TEXT, 
                thumb   TEXT,
                duration INT,
                bitrate FLOAT,
                PRIMARY KEY(audio_id AUTOINCREMENT)
            )", [],
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create table indices: {}", error);
                return Err(error);
            }
        }
        match connection.execute("
            CREATE TABLE albums (
                album_id INTEGER, 
                album_name TEXT, 
                PRIMARY KEY(album_id AUTOINCREMENT)
            )", [],
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create table parameters: {}", error);
                return Err(error);
            }
        }
        match connection.execute(
            "CREATE INDEX index_albums_name ON albums (album_name)", (),
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create index on albums: {}", error);
                return Err(error);
            }
        }
        match connection.execute("
        CREATE TABLE album_audio_map (
            entry_id INTEGER,
            audio_id INTEGER,
            album_id INTEGER,
            PRIMARY KEY(entry_id AUTOINCREMENT) 
            )", [],
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create table albums: {}", error);
                return Err(error);
            }
        }
        match connection.execute(
            "CREATE INDEX index_album_audio_audio_id ON album_audio_map (audio_id)", (),
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create index on albums: {}", error);
                return Err(error);
            }
        }

        match connection.execute("
            CREATE TABLE artists (
                artist_id INTEGER, 
                artist_name TEXT, 
                PRIMARY KEY(artist_id AUTOINCREMENT)
            )", [],
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create table artists: {}", error);
                return Err(error);
            }
        }
        match connection.execute(
            "CREATE INDEX index_artists_name_id ON artists (artist_name)", (),
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create index on artists: {}", error);
                return Err(error);
            }
        }
        match connection.execute("
        CREATE TABLE artist_audio_map (
            entry_id INTEGER,
            audio_id INTEGER,
            artist_id INTEGER, 
            PRIMARY KEY(entry_id AUTOINCREMENT)
            )", [],
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create table artist_audio_map: {}", error);
                return Err(error);
            }
        }
        match connection.execute(
            "CREATE INDEX index_artist_audio_map_audio_id ON artist_audio_map (audio_id)", (),
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create index on artist_audio_map: {}", error);
                return Err(error);
            }
        }
 
        match connection.execute("
        CREATE TABLE albumartist_audio_map (
            entry_id INTEGER,
            audio_id INTEGER,
            albumartist_id INTEGER, 
            PRIMARY KEY(entry_id AUTOINCREMENT)
            )", [],
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create table albumartist_audio_map: {}", error);
                return Err(error);
            }
        }
        match connection.execute(
            "CREATE INDEX index_albumartist_audio_map_audio_id ON artist_audio_map (audio_id)", (),
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create index on albumartist_audio_map: {}", error);
                return Err(error);
            }
        }
        match connection.execute("
            CREATE TABLE indexes (
                index_id INTEGER, 
                audio_id INTEGER, 
                title TEXT, 
                start DOUBLE,
                end DOUBLE,
                PRIMARY KEY(index_id AUTOINCREMENT)
            )", [],
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create table parameters: {}", error);
                return Err(error);
            }
        }
        match connection.execute(
            "CREATE INDEX index_indexes_audio_id ON indexes (audio_id)", (),
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create index on artist_audio_map: {}", error);
                return Err(error);
            }
        }
        match connection.execute("
            CREATE TABLE lyrics (
                lyrics_id INTEGER, 
                audio_id INTEGER, 
                lyricsfile TEXT, 
                PRIMARY KEY(lyrics_id AUTOINCREMENT)
            )", [],
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create table parameters: {}", error);
                return Err(error);
            }
        }
        match connection.execute(
            "CREATE INDEX index_lyrics_audio_id ON lyrics (audio_id)", (),
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create index on lyrics: {}", error);
                return Err(error);
            }
        }

        match connection.execute("
            CREATE TABLE image_metadata (
                image_id INTEGER,
                name     TEXT NOT NULL, 
                path     TEXT NOT NULL, 
                created  UNSIGNED BIG INT NOT NULL, 
                resized  TEXT, 
                thumb    TEXT,
                width    INTEGER,
                height   INTEGER,
                Photographer TEXT,
                LenseModel TEXT,
                Focallength TEXT,
                Exposuretime TEXT,
                FNumber  TEXT,
                GPSString  TEXT,
                GPSLatitude DOUBLE,
                GPSLongitude DOUBLE,
                GPSAltitude DOUBLE,
                PRIMARY KEY(image_id AUTOINCREMENT)
            )", [],
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create table image_metadata: {}", error);
                return Err(error);
            }
        }
        match connection.execute(
            "CREATE INDEX index_image_photographer ON image_metadata (Photographer)", (),
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create index on image_metadata: {}", error);
                return Err(error);
            }
        }
        match connection.execute(
            "CREATE INDEX index_image_name ON image_metadata (name)", (),
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create index on image_metadata: {}", error);
                return Err(error);
            }
        }
        match connection.execute("
            CREATE TABLE searches (
                search_id INTEGER,
                search_string  TEXT, 
                from_string  TEXT, 
                from_value DOUBLE, 
                to_string  TEXT, 
                to_value DOUBLE, 
                image  INTEGER, 
                video  INTEGER, 
                audio  INTEGER, 
                filepath  INTEGER, 
                title  INTEGER, 
                description  INTEGER, 
                actor  INTEGER, 
                director  INTEGER, 
                artist  INTEGER, 
                album_artist  INTEGER, 
                duration  INTEGER, 
                creation_date  INTEGER, 
                modification_date  INTEGER, 
                release_date  INTEGER, 
                lense_model  INTEGER, 
                focal_length  INTEGER, 
                exposure_time  INTEGER, 
                fnumber  INTEGER, 
                gps_latitude  INTEGER, 
                gps_longitude  INTEGER, 
                gps_altitude  INTEGER, 
                PRIMARY KEY(search_id AUTOINCREMENT)
            )", [],
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create table image_metadata: {}", error);
                return Err(error);
            }
        }
} else {
        connection = Connection::open(sqlite_file)?;
    }
    Ok(connection)
}
