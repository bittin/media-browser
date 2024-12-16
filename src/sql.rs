use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use rusqlite::{Connection, Result, params};
use std::path::PathBuf;
use std::collections::BTreeMap;
use chrono::NaiveDate;

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct VideoMetadata {
    pub name: String,
    pub title: String,
    pub date: NaiveDate,
    pub path: String,
    pub poster: String,
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
}

impl Default for VideoMetadata {
    fn default() -> VideoMetadata {
        VideoMetadata {
            name: String::new(),
            title: String::new(),
            date: NaiveDate::from_ymd_opt(1970, 1,1).unwrap(),
            path: String::new(),
            poster: String::new(),
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
        "INSERT INTO video_metadata (name, title, released, poster, duration, width, height, framerate, description) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![&metadata.name, &metadata.title, &metadata.date, &metadata.poster, &metadata.duration, &metadata.width, &metadata.height, &metadata.framerate, &metadata.description],
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
                        let s_opt = row.get(1);
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

    let mut creationtime: u64 = 0;
    if let Ok(created) = statdata.created() {
        match created.duration_since(std::time::UNIX_EPOCH) {
            Ok(n) => creationtime = n.as_secs(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        }
    }
    let mut modifiedtime: u64 = 0;
    if let Ok(modified) = statdata.modified() {
        match modified.duration_since(std::time::UNIX_EPOCH) {
            Ok(n) => modifiedtime = n.as_secs(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        }
    }
    match connection.execute(
        "INSERT INTO file_metadata (filepath, creation_time, modificattion_time, file_type, video_id) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![&metadata.path, &creationtime, &modifiedtime, &2, &video_id],
    ) {
        Ok(_retval) => {}, //log::warn!("Inserted {} video with ID {} and location {} into candidates.", video.id, video.index, candidate_id),
        Err(error) => {
            log::error!("Failed to insert video into  database: {}", error);
            return;
        }
    }
    let meta = FileMetadata {
        filepath: PathBuf::from(&metadata.path),
        creation_time: creationtime,
        modification_time: modifiedtime,
        file_type: 2,
        metadata_id: video_id,
        ..Default::default()
    };
    known_files.insert(meta.filepath.clone(), meta);

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
    for i in 0..metadata.director.len() {
        match connection.execute(
            "INSERT INTO directors (video_id, director_name) VALUES (?1, ?2)",
            params![&video_id, &metadata.director[i]],
        ) {
            Ok(_retval) => {}, //log::warn!("Inserted {} video with ID {} and location {} into candidates.", video.id, video.index, candidate_id),
            Err(error) => {
                log::error!("Failed to insert video into  database: {}", error);
                return;
            }
        }
    }
    for i in 0..metadata.actors.len() {
        match connection.execute(
            "INSERT INTO actors (video_id, actor_name) VALUES (?1, ?2)",
            params![&video_id, &metadata.actors[i]],
        ) {
            Ok(_retval) => {}, //log::warn!("Inserted {} video with ID {} and location {} into candidates.", video.id, video.index, candidate_id),
            Err(error) => {
                log::error!("Failed to insert video into  database: {}", error);
                return;
            }
        }
    }

}

pub fn delete_video(
    connection: &mut rusqlite::Connection, 
    metadata: &mut crate::sql::VideoMetadata,
    known_files: &mut std::collections::BTreeMap<PathBuf, crate::sql::FileMetadata>,
) {
    // Get the index.
    //let index = self.ids[id];
    let mut video_id: u32 = 0;
    let query = "SELECT video_id FROM file_metadata WHERE filepath = ?1";
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

pub fn video(
    connection: &mut rusqlite::Connection, 
    filepath: &str,
    known_files: &mut std::collections::BTreeMap<PathBuf, crate::sql::FileMetadata>,
) -> VideoMetadata {
    let mut v = VideoMetadata {..Default::default()};
    v.path = filepath.to_string();
    let filedata = file(connection, filepath);
    let video_id = filedata.metadata_id;
    let query = "SELECT name, title, released, poster, duration, width, height, framerate, description FROM video_metadata WHERE video_id = ?1";
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
    let query = "SELECT director_name FROM directors WHERE video_id = ?1";
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
    let query = "SELECT actor_name FROM actors WHERE video_id = ?1";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![&video_id]) {
                Ok(mut rows) => {
                    loop {
                        match rows.next() {
                            Ok(Some(row)) => {
                                match row.get::<usize, String>(0) {
                                    Ok(val) => v.subtitles.push(val),
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

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct AudioMetadata {
    pub name: String,
    pub title: String,
    pub date: NaiveDate,
    pub path: String,
    pub poster: String,
    pub duration: u32,
    pub bitrate: f32,
    pub album: String,
    pub artist: Vec<String>,
    pub albumartist: Vec<String>,
}

impl Default for AudioMetadata {
    fn default() -> AudioMetadata {
        AudioMetadata {
            name: String::new(),
            title: String::new(),
            date: NaiveDate::from_ymd_opt(1970, 1, 1).unwrap(),
            path: String::new(),
            poster: String::new(),
            duration: 0,
            bitrate: 0.0,
            album: String::new(),
            artist: Vec::new(),
            albumartist: Vec::new(),
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
        "INSERT INTO audio_metadata (name, title, released, poster, duration, bitrate) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![&metadata.name, &metadata.title, &metadata.date, &metadata.poster, &metadata.duration, &metadata.bitrate],
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
                        let s_opt = row.get(1);
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

    let mut creationtime: u64 = 0;
    if let Ok(creatted) = statdata.created() {
        match creatted.duration_since(std::time::UNIX_EPOCH) {
            Ok(n) => creationtime = n.as_secs(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        }
    }
    let mut modifiedtime: u64 = 0;
    if let Ok(modified) = statdata.modified() {
        match modified.duration_since(std::time::UNIX_EPOCH) {
            Ok(n) => modifiedtime = n.as_secs(),
            Err(_) => panic!("SystemTime before UNIX EPOCH!"),
        }
    }
    match connection.execute(
        "INSERT INTO file_metadata (filepath, creation_time, modificattion_time, file_type, metadata_id) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![&metadata.path, &creationtime, &modifiedtime, &3, &audio_id],
    ) {
        Ok(_retval) => {}, //log::warn!("Inserted {} video with ID {} and location {} into candidates.", video.id, video.index, candidate_id),
        Err(error) => {
            log::error!("Failed to insert file into  database: {}", error);
            return;
        }
    }
    let meta = FileMetadata {
        filepath: PathBuf::from(&metadata.path),
        creation_time: creationtime,
        modification_time: modifiedtime,
        file_type: 2,
        metadata_id: audio_id,
        ..Default::default()
    };
    known_files.insert(meta.filepath.clone(), meta.clone());
    let thumbpath = PathBuf::from(&metadata.path);
    known_files.insert(thumbpath.join(".png"), meta);
    for i in 0..metadata.artist.len() {
        match connection.execute(
            "INSERT INTO artists (audio_id, artist_name) VALUES (?1, ?2)",
            params![&audio_id, &metadata.artist[i]],
        ) {
            Ok(_retval) => {}, //log::warn!("Inserted {} video with ID {} and location {} into candidates.", video.id, video.index, candidate_id),
            Err(error) => {
                log::error!("Failed to insert audio {} into  database: {}", audio_id, error);
                return;
            }
        }
    }
    match connection.execute(
        "INSERT INTO albums (audio_id, album_name) VALUES (?1, ?2)",
        params![&audio_id, &metadata.album],
    ) {
        Ok(_retval) => {}, //log::warn!("Inserted {} video with ID {} and location {} into candidates.", video.id, video.index, candidate_id),
        Err(error) => {
            log::error!("Failed to insert audio {} into  database: {}", metadata.album, error);
            return;
        }
    }
    let mut album_id: u32 = 0;
    let query = "SELECT last_insert_rowid()";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![]) {
                Ok(mut rows) => {
                    while let Ok(Some(row)) = rows.next() {
                        let s_opt = row.get(1);
                        if s_opt.is_ok() {
                            album_id = s_opt.unwrap();
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
    for i in 0..metadata.albumartist.len() {
        match connection.execute(
            "INSERT INTO album_artists (audio_id, album_id, albumartist_name) VALUES (?1, ?2, ?3)",
            params![&audio_id, &album_id, &metadata.albumartist[i]],
        ) {
            Ok(_retval) => {}, //log::warn!("Inserted {} video with ID {} and location {} into candidates.", video.id, video.index, candidate_id),
            Err(error) => {
                log::error!("Failed to insert albumartist {} into  database: {}", metadata.albumartist[i], error);
                return;
            }
        }
    }


}

pub fn delete_audio(
    connection: &mut rusqlite::Connection, 
    metadata: &mut AudioMetadata,
    known_files: &mut std::collections::BTreeMap<PathBuf, crate::sql::FileMetadata>,
) {
    // Get the index.
    //let index = self.ids[id];
    let mut audio_id: u32 = 0;
    let query = "SELECT video_id FROM file_metadata WHERE filepath = ?1";
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
    // clear the entry in the candidates list without deleting it
    let ret = connection.execute(
        "DELETE FROM albums WHERE audio_id = ?1",
        params![&audio_id],
    );
    if ret.is_err() {
        log::error!("Failed to delete albums {}!", audio_id);
        return;
    }    
    // clear the entry in the candidates list without deleting it
    let ret = connection.execute(
        "DELETE FROM album_artists WHERE audio_id = ?1",
        params![&audio_id],
    );
    if ret.is_err() {
        log::error!("Failed to delete album_artists {}!", audio_id);
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
    let query = "SELECT name, title, released, poster, duration, bitrate FROM audio_metadata WHERE audio_id = ?1";
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
                                    Ok(val) => v.duration = val,
                                    Err(error) => {
                                        log::error!("Failed to read duration for audio: {}", error);
                                        continue;
                                    }
                                }
                                match row.get(4) {
                                    Ok(val) => v.bitrate = val,
                                    Err(error) => {
                                        log::error!("Failed to read bitrate for audio: {}", error);
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
    let query = "SELECT artist_name FROM artists WHERE audio_id = ?1";
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
    let query = "SELECT album_name FROM albums WHERE audio_id = ?1";
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
    let query = "SELECT albumartist_name FROM album_artists WHERE audio_id = ?1";
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
                                        log::error!("Failed to read album_artists for video: {}", error);
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
    let mut v = BTreeMap::new();
    let query = "SELECT * FROM file_metadata";
    match connection.prepare(query) {
        Ok(mut statement) => {
            match statement.query(params![]) {
                Ok(mut rows) => {
                    loop {
                        let mut s = FileMetadata {..Default::default()};
                        match rows.next() {
                            Ok(Some(row)) => {
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
                                v.insert(s.filepath.clone(), s);
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

pub fn connect() -> Result<rusqlite::Connection, rusqlite::Error> {
    let sqlite_file;
    let mut connection;
    match dirs::data_local_dir() {
        Some(pb) => {
            let mut dir = pb.join("cosmic-media-browser");
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
        match connection.execute(
            "CREATE TABLE file_metadata (
                filepath TEXT NOT NULL unique PRIMARY KEY NOT NULL, 
                creation_time UNSIGNED BIG INT, 
                modificattion_time UNSIGNED BIG INT, 
                file_type INT,   // 0 other file, 1 image, 2, video, 3 audio
                metadata_id BIG INT
            )", (),
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create table file_metadata: {}", error);
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
        match connection.execute("
            CREATE TABLE subtitles (
                subtitle_id INTEGER, 
                video_id BIG INT, 
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
                video_id BIG INT, 
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
                video_id BIG INT, 
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
            CREATE TABLE directors (
                director_id INTEGER, 
                video_id BIG INT, 
                director_name TEXT, 
                PRIMARY KEY(director_id AUTOINCREMENT)
            )", [],
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create table parameters: {}", error);
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
                actor_id INTEGER, 
                video_id BIG INT, 
                actor_name TEXT, 
                PRIMARY KEY(actor_id AUTOINCREMENT)
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
            CREATE TABLE artists (
                artist_id INTEGER, 
                audio_id BIG INT, 
                artist_name TEXT, 
                PRIMARY KEY(artist_id AUTOINCREMENT)
            )", [],
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create table parameters: {}", error);
                return Err(error);
            }
        }
        match connection.execute(
            "CREATE INDEX index_artists_audio_id ON artists (audio_id)", (),
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create index on artists: {}", error);
                return Err(error);
            }
        }
        match connection.execute("
            CREATE TABLE albums (
                album_id INTEGER, 
                audio_id BIG INT, 
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
            "CREATE INDEX index_albums_audio_id ON albums (audio_id)", (),
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create index on albums: {}", error);
                return Err(error);
            }
        }
        match connection.execute("
            CREATE TABLE album_artists (
                albumartist_id INTEGER, 
                audio_id BIG INT, 
                album_id BIG INT,
                albumartist_name TEXT, 
                PRIMARY KEY(albumartist_id AUTOINCREMENT)
            )", [],
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create table parameters: {}", error);
                return Err(error);
            }
        }
        match connection.execute(
            "CREATE INDEX index_album_artists_audio_id ON album_artists (audio_id)", (),
        ) {
            Ok(_ret) => {},
            Err(error) => {
                log::error!("Failed to create index on album_artists: {}", error);
                return Err(error);
            }
        }
 
    } else {
        connection = Connection::open(sqlite_file)?;
    }
    Ok(connection)
}
