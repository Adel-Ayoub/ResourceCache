use std::{
    collections::HashMap,
    fmt::Display,
    fs,
    io::{self, Write, prelude::*},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH, Duration},
};

use anyhow::Result;

use bytes::Bytes;
use config::Config;
use rand::{thread_rng, Rng};
use reqwest::IntoUrl;
use serde::{Deserialize, Serialize};
use human_bytes::human_bytes;

pub mod cli;
pub mod config;


#[derive(Serialize, Deserialize)]
pub struct CachedFile {
    pub url: String,
    pub path: String,
    pub cached_time: u64, //UNIX_APOCH time
    pub random: i32,      //random offset in cache lifetime
    pub expire_time: Option<u64> //UNIX_APOCH time
}


pub struct Cacher {
    config: Config,
    file_map: HashMap<String, CachedFile>,
}

impl Cacher {
    pub fn new(config: Option<Config>) -> Cacher {
        let config = config.unwrap_or_default();
        let map = Cacher::load(&config.cache_json);
        Cacher {
            config,
            file_map: map,
        }
    }

    pub fn stats(&mut self) -> Stats {
        let mut number_of_cached_files: i32 = 0;
        let mut max_file_size: (String, u64) = ("".to_string(), 0);
        let mut min_file_size: (String, u64) = ("".to_string(), u64::MAX);
        let mut total_size: u64 = 0;
        let mut last_to_expire: (String, u64) = ("".to_string(), 0);
        let mut first_to_expire: (String, u64) = ("".to_string(), u64::MAX);
        self.clean_expired();
        for cached_file in self.file_map.values() {
            if let Ok(metadata) = fs::metadata(&cached_file.path) {
                if metadata.is_file() {
                    number_of_cached_files += 1;
                    if metadata.len() > max_file_size.1 {
                        max_file_size = (cached_file.path.clone(), metadata.len());
                    }
                    if metadata.len() < min_file_size.1 {
                        min_file_size = (cached_file.path.clone(), metadata.len());
                    }
                    total_size += metadata.len();
                    let expire_time =
                        Cacher::get_expire_time(cached_file, self.config.file_lifetime);
                    if expire_time > last_to_expire.1 {
                        last_to_expire = (cached_file.path.clone(), expire_time);
                    }
                    if expire_time < first_to_expire.1 {
                        first_to_expire = (cached_file.path.clone(), expire_time);
                    }
                }
            }
        }

        Stats {
            first_to_expire,
            number_of_cached_files,
            max_file_size,
            min_file_size,
            total_size,
            last_to_expire,
        }
    }

    pub fn save(&self) -> Result<()> {
        let text = serde_json::to_string_pretty(&self.file_map)?;
        fs::write(&self.config.cache_json, text)?;
        Ok(())
    }

    fn load<P: AsRef<Path>>(path: P) -> HashMap<String, CachedFile> {
        let mut tmp=fs::OpenOptions::new().read(true).write(true).create(true).open(&path).unwrap();
        if tmp.metadata().unwrap().len() == 0 {
            drop(tmp);
            tmp=fs::OpenOptions::new().read(true).create(false).open(&path).unwrap();
        }
        tmp.sync_all().expect("Error syncing cache file");
        let mut content=String::new();
        match tmp.read_to_string(&mut content) {
            Ok(size) => {
                if size==0 {
                    writeln!(io::stderr(), "error reading file: 0 size")
                            .expect("printing to stderr failed");
                }
                let map: Result<HashMap<String, CachedFile>, serde_json::Error> =
                    serde_json::from_str(&content);
                match map {
                    Ok(file_map) => file_map,
                    Err(err) => {
                        writeln!(io::stderr(), "cahce corrupted {}", err)
                            .expect("printing to stderr failed");
                        HashMap::new()
                    }
                }
            }
            Err(err) => {
                writeln!(io::stderr(), "cannot open file {}", err)
                    .expect("printing to stderr failed");
                panic!();
            },
        }
        
    }

    fn init_path(&self) -> Result<()> {
        fs::create_dir_all(&self.config.cache_dir)?;
        Ok(())
    }

    pub fn clean_expired(&mut self) -> u16 {
        let now = get_now_unix_epoch();
        let mut to_remove: Vec<String> = Vec::new();
        for key in self.file_map.keys() {
            let cached_file = self.file_map.get(key).expect("shouldn't happen");
            let expire_time = Cacher::get_expire_time(cached_file, self.config.file_lifetime);
            if expire_time < now {
                to_remove.push(key.clone());
            }
        }
        let mut count = 0;
        for key in to_remove {
            let cached_file = self.file_map.remove(&key).expect("shouldn't happen");
            if let Err(err) = fs::remove_file(&cached_file.path) {
                writeln!(io::stderr(), "{},{}", err, cached_file.path)
                    .expect("printing to stderr failed");
            }
            count+=1;
        }
        self.save().expect("failed to save cache-db");
        count
    }

    fn get_expire_time(cached_file: &CachedFile, file_lifetime: u64) -> u64 {
        match cached_file.random < 0 {
            true => {
                cached_file.cached_time + file_lifetime - cached_file.random.unsigned_abs() as u64
            }
            false => cached_file.cached_time + file_lifetime + cached_file.random as u64,
        }
    }

    pub async fn get<T: IntoUrl>(
        &mut self,
        url: T,
        filename: &str,
        refresh: bool,
        expire_time: Option<u64>,
    ) -> Result<String> {
        self.clean_expired();
        if !refresh {
            if let Some(cached_file) = self.file_map.get(url.as_str()) {
                let path = PathBuf::from(&cached_file.path);
                if path.exists() && path.is_file() {
                    return Ok(cached_file.path.clone());
                }
            }
        }
        self.file_map.remove(url.as_str());

        let bytes = Cacher::get_from_url(url.as_str()).await?;
        let mut counter = 0;
        let filenames: Vec<String> = self
            .file_map
            .values()
            .map(|cached_file| {
                Path::new(&cached_file.path)
                    .file_name()
                    .expect("corrupted cache")
                    .to_str()
                    .expect("path wasn't valid UTF-8")
                    .to_string()
            })
            .collect();
        let mut filename_test = filename.to_owned();
        while filenames.contains(&filename_test) {
            counter += 1;
            filename_test = counter.to_string() + filename;
        }

        let mut path = self.config.cache_dir.clone();
        path.push(filename_test);
        self.init_path()?;
        fs::write(&path, bytes)?;
        let path = path.to_str().expect("path wasn't valid UTF-8").to_owned();
        let mut rng = thread_rng();
        let mut expiration=Option::<u64>::None;
        if let Some(expire_time) = expire_time{
            expiration=Some(get_now_unix_epoch() + expire_time);
        }
        let cached_file = CachedFile {
            path,
            url: url.as_str().to_owned(),
            cached_time: get_now_unix_epoch(),
            random: rng.gen_range(self.config.default_offset_range.clone()),
            expire_time: expiration
        };
        self.file_map.insert(url.as_str().to_owned(), cached_file);

        return Ok(self.file_map.get(url.as_str()).expect("should be there because it was just inserted").path.to_owned());
    }

    async fn get_from_url<T: IntoUrl>(url: T) -> reqwest::Result<Bytes> {
        reqwest::get(url).await?.bytes().await
    }

    pub fn clear(&mut self) -> Result<i16> {
        let mut count=0;
        for path in self.file_map.values().map(|v| &v.path) {
            if let Err(err) = fs::remove_file(path) {
                writeln!(io::stderr(), "{},{}", err, path).expect("printing to stderr failed");
            }
            count+=1;
        }
        self.file_map.clear();
        self.save()?;
        Ok(count)
    }
}

pub struct Stats {
    pub number_of_cached_files: i32,
    pub max_file_size: (String, u64),
    pub min_file_size: (String, u64),
    pub total_size: u64,
    pub first_to_expire: (String, u64),
    pub last_to_expire: (String, u64),
}

impl Display for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut buf = String::new();
        buf.push_str(&format!(
            "number of files in cache: {}\n",
            self.number_of_cached_files
        ));
        buf.push_str(&format!(
            "largest file: {}\n\tsize: {}\n",
            self.max_file_size.0, human_bytes(self.max_file_size.1 as f64)
        ));
        buf.push_str(&format!(
            "smallest file: {}\n\tsize: {}\n",
            self.min_file_size.0, human_bytes(self.min_file_size.1 as f64)
        ));
        buf.push_str(&format!("total cache size: {}\n", human_bytes(self.total_size as f64)));
        buf.push_str(&format!(
            "next file to expire: {}\n\t{}\n",
            self.first_to_expire.0,
            format_unix_epoch_duration(&self.first_to_expire.1)
        ));
        buf.push_str(&format!(
            "last file to expire: {}\n\t{}\n",
            self.last_to_expire.0,
            format_unix_epoch_duration(&self.last_to_expire.1)
        ));
        write!(f, "{}", buf)
    }
}

fn format_unix_epoch_duration(time: &u64) -> String {
    let mut fmt=timeago::Formatter::new();
    fmt.ago("").min_unit(timeago::TimeUnit::Seconds).max_unit(timeago::TimeUnit::Days).num_items(2);
    
    let time: i64 = *time as i64
        - get_now_unix_epoch() as i64;
        let time_str = fmt.convert(Duration::from_secs(time.unsigned_abs()));
    if time < 0 {
        format!("{} ago", time_str)
    } else {
        format!("in {}", time_str)
    }
}

fn get_now_unix_epoch() -> u64 {
    SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs()
}