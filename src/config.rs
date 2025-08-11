use std::{path::{PathBuf, Component}, ops::Range, env, collections::HashMap, fs::{self, File}, io::Write};
use anyhow::{Result, Context, bail};

pub struct Config {
    pub cache_json: PathBuf,
    pub cache_dir: PathBuf,
    pub default_offset_range: Range<i32>,
    pub file_lifetime: u64,
}

const CONFIG_DEFAULT_CONTENTS: &str = r#"
cache-db = "$HOME/.cache/resource-cache/cache.json"
cahe-dir = "$HOME/.cache/resource-cache/cache/"
random-offset-range = "-36000..36001"
file-default-lifetime = 1728000
"#;

impl Default for Config {
    fn default() -> Self {
        Config {
            cache_json: PathBuf::from(get_absolute_path_with_variables(
                "$HOME/.cache/resource-cache/cache.json",
            )),
            cache_dir: PathBuf::from(get_absolute_path_with_variables(
                "$HOME/.cache/resource-cache/cache/",
            )),
            default_offset_range: -36000..36001,
            file_lifetime: 1728000,
        }
    }
}

impl Config{
    pub fn config_from_hashmap(map: HashMap<String, String>) -> Config {
        let mut conf = Config::default();

        if let Some(value) = map.get("cache-db") {
            conf.cache_json = value.parse().unwrap_or_else(|_| conf.cache_json.clone());
        }

        if let Some(value) = map.get("cache-dir") {
            conf.cache_dir = value.parse().unwrap_or_else(|_| conf.cache_dir.clone());
        }

        if let Some(value) = map.get("random-offset-range") {
            if let Ok(value) = parse_range(value) {
                conf.default_offset_range = value;
            }
        }

        if let Some(value) = map.get("file-default-lifetime") {
            conf.file_lifetime = value.parse().unwrap_or_else(|_| conf.file_lifetime);
        }

        conf
    }

    pub fn read_config(path: &str) -> Config {
        let file_string = get_absolute_path_with_variables(path);
        let file_bytes = match fs::read(file_string) {
            Ok(file) => file,
            Err(err) => panic!("error reading config file: {err}"),
        };

        let mut entries = props_rs::parse(file_bytes.as_slice())
            .unwrap_or_else(|err| panic!("failed to parse config file {}, err:{}", path, err));

        for entry in &mut entries {
            entry.key = entry.key.to_lowercase();
            if let Some(value) = entry.value.strip_prefix('\"') {
                let value_stripped = value.strip_suffix('\"').expect("config file malformed: missing \" at the end of the line").to_owned();
                entry.value = get_absolute_path_with_variables(&value_stripped);
            }
        }

        let entries = props_rs::to_map(entries);

        Config::config_from_hashmap(entries)
    }

    pub fn ensure_create_new_file(path: &str) -> Result<()> {
        let path = get_absolute_path_with_variables(path);
        let mut dir_path=std::path::PathBuf::from(&path);
        dir_path.pop();
        fs::create_dir_all(dir_path)?;
        if let Ok(mut file) = File::options().create_new(true).write(true).open(path) {
            file.write_all(CONFIG_DEFAULT_CONTENTS.as_bytes())?;
            file.flush()?;
        }
        Ok(())
    }
}


fn parse_range(s: &str) -> Result<Range<i32>> {
    if s.matches("..").count() != 1 {
        bail!("error parsing range: multiple occourrences or missing '..'")
    }
    let vec: Vec<_> = s.split("..").collect();
    let (from, to) = (vec.first().context("failed to finding from string")?, vec.get(1).context("error finding to string")?);
    let (from, to) = (from.parse::<i32>().context("failed to parse from")?, to.parse::<i32>().context("error parsing to")?);
    Ok(Range { start: from, end: to})
}

pub fn get_absolute_path_with_variables(path: &str) -> String {
    let mut final_path: PathBuf = PathBuf::new();
    
    // First, expand environment variables
    let mut expanded_path = path.to_string();
    for component in path.split('/') {
        if component.starts_with('$') {
            let var_name = &component[1..]; // Remove the $ prefix
            if let Ok(var_value) = env::var(var_name) {
                expanded_path = expanded_path.replace(component, &var_value);
            }
        }
    }
    
    // Now try to canonicalize the expanded path
    let absolute_path = PathBuf::from(&expanded_path)
        .canonicalize()
        .unwrap_or_else(|_| PathBuf::from(&expanded_path));
    
    // Build the final path
    for component in absolute_path.components() {
        let component = component.as_os_str().to_str().expect("not valid UTF-8");
        if let Some(var) = component.strip_prefix('$') {
            let var = env::var(var).unwrap_or("".to_string());
            for component in PathBuf::from(var).components() {
                if let Component::RootDir = component {
                    if final_path.components().count() != 0 {
                        continue;
                    }
                }
                final_path.push(component);
            }
        } else {
            final_path.push(component);
        }
    }

    final_path.to_str().unwrap_or(&expanded_path).to_string()
}
