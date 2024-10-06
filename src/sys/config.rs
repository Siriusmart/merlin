use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    time::UNIX_EPOCH,
};

use jsonc_to_json::jsonc_to_json;
use serde::{de::DeserializeOwned, Serialize};

pub trait Config: Serialize + DeserializeOwned + Default {
    const NAME: &'static str;
    const NOTE: &'static str = "";

    fn path() -> PathBuf {
        dirs::config_dir()
            .unwrap()
            .join(env!("CARGO_PKG_NAME"))
            .join(format!("{}.jsonc", Self::NAME))
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
            serde_json::to_string(self).unwrap()
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
                Ok(val) => return val,
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
                    return out;
                }
            }
        }

        let out = Self::default();
        out.save();
        out
    }
}
