use std::{
    collections::HashMap,
    fs::{self, OpenOptions},
    hash::{DefaultHasher, Hash, Hasher},
    io::Write,
    path::PathBuf,
    sync::OnceLock,
    time::UNIX_EPOCH,
};

use jsonc_to_json::jsonc_to_json;
use serde::{de::DeserializeOwned, Serialize};

static mut CONFIG_HASHES: OnceLock<HashMap<String, u64>> = OnceLock::new();

pub trait Config: Serialize + DeserializeOwned + Default + Hash {
    const NAME: &'static str;
    const NOTE: &'static str = "";

    fn path() -> PathBuf {
        dirs::config_dir()
            .unwrap()
            .join(env!("CARGO_PKG_NAME"))
            .join(format!("{}.jsonc", Self::NAME))
    }

    fn smart_save(&self) {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);

        let hashes = unsafe { CONFIG_HASHES.get_mut() }.unwrap();
        let old = hashes.get_mut(Self::NAME).unwrap();
        let new = hasher.finish();

        if *old != new {
            self.save();
            *old = new;
        }
    }

    fn save(&self) {
        let ser = format!(
            "{}{}",
            if Self::NOTE.is_empty() {
                "".to_string()
            } else {
                format!(
                    "{}\n\n",
                    Self::NOTE
                        .lines()
                        .map(|line| format!("// {line}"))
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            },
            serde_json::to_string_pretty(self).unwrap()
        );

        let path = Self::path();

        fs::create_dir_all(path.parent().unwrap()).unwrap();

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .unwrap();

        file.write_all(ser.as_bytes()).unwrap();
    }

    fn load() -> Self {
        let path = Self::path();

        if path.exists() {
            let content = fs::read_to_string(&path).unwrap();
            let json = jsonc_to_json(&content);
            match serde_json::from_str(&json) {
                Ok(val) => return insert_hash(val),
                Err(_) => {
                    let meta = path
                        .metadata()
                        .unwrap()
                        .created()
                        .unwrap()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs();
                    fs::rename(&path, path.with_file_name(format!("{}-{meta}", Self::NAME)))
                        .unwrap();
                    let out = Self::default();
                    out.save();
                    return insert_hash(out);
                }
            }
        }

        let out = Self::default();
        out.save();
        insert_hash(out)
    }
}

fn insert_hash<C: Config>(config: C) -> C {
    if unsafe { CONFIG_HASHES.get() }.is_none() {
        let _ = unsafe { CONFIG_HASHES.set(HashMap::new()) };
    }

    let mut hasher = DefaultHasher::new();
    config.hash(&mut hasher);

    unsafe { CONFIG_HASHES.get_mut() }
        .unwrap()
        .insert(C::NAME.to_string(), hasher.finish());
    config
}
