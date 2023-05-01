use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::sync::{RwLock, Arc};
use std::{collections::HashMap, fs::File, path::Path};

use directories::ProjectDirs;

const APP_NAME: &str = "draft-claw";
const APP_AUTHOR: &str = "akio";
const APP_QUALIFIER: &str = "com";

const CONFIG_FILE_NAME: &str = "runtime_data.json";

pub struct Context {
    pub data: Arc<RwLock<HashMap<String, String>>>,
    config_file_path: String,
}

pub fn create_context() -> Context {
    let project_dirs = ProjectDirs::from(APP_QUALIFIER, APP_AUTHOR, APP_NAME)
        .expect("Failed to get the project directory");

    let runtime_dir = project_dirs
        .data_local_dir();
    println!("Runtime directory: {:?}", runtime_dir);

    std::fs::create_dir_all(&runtime_dir).expect("Failed to create the runtime directory");

    let config_file_path = runtime_dir.join(CONFIG_FILE_NAME);
    load_data_from_file(config_file_path.to_str().unwrap())
}

pub fn load_data_from_file(config_file_path: &str) -> Context {
    let path = Path::new(config_file_path);
    let data = if path.exists() {
        let mut file = File::open(path).expect("Failed to open file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Failed to read file");
        let data: HashMap<String, String> =
            serde_json::from_str(&contents).expect("Failed to parse JSON");
        data
    } else {
        HashMap::new()
    };

    Context {
        data: Arc::new(RwLock::new(data)),
        config_file_path: config_file_path.to_string(),
    }
}

impl Context {
    pub fn read_data(&self, key: &str) -> Option<String> {
        let data_read = self.data.read().unwrap();
        data_read.get(key).map(|s| s.to_string())
    }

    pub fn write_data(&self, key: &str, value: &str) {
        let mut data_write = self.data.write().unwrap();
        data_write.insert(key.to_string(), value.to_string());
        save_data(&self.config_file_path, &data_write);
    }
}

fn save_data(config_file_path: &str, data: &HashMap<String, String>) {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(config_file_path)
        .expect("Failed to create or open the file");
    let content = serde_json::to_string(data).expect("Failed to serialize the data");
    file.write_all(content.as_bytes())
        .expect("Failed to write to the file");
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn create_temp_file_path() -> String {
        NamedTempFile::new()
            .expect("Failed to create a temp file")
            .path()
            .to_str()
            .unwrap()
            .to_string()
    }

    #[test]
    fn test_read_write_data() {
        let config_file_path = create_temp_file_path();
        let runtime_data = load_data_from_file(&config_file_path);

        assert_eq!(runtime_data.read_data("key"), None);

        runtime_data.write_data("key", "value");
        assert_eq!(runtime_data.read_data("key"), Some("value".to_string()));
    }

    #[test]
    fn test_persistence() {
        let config_file_path = create_temp_file_path();

        {
            let runtime_data = load_data_from_file(&config_file_path);
            runtime_data.write_data("key", "value");
        }

        {
            let runtime_data = load_data_from_file(&config_file_path);
            assert_eq!(runtime_data.read_data("key"), Some("value".to_string()));
        }
    }
}
