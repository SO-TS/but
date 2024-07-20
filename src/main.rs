use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::exit;
use std::time::SystemTime;

use log::*;
use serde::{Deserialize, Serialize};

use std::io::Seek;
use std::path::Path;
use std::str;
 
use walkdir::{DirEntry, WalkDir};
use zip::write::FileOptions;


// 参考：https://blog.csdn.net/u013195275/article/details/105973490
fn compress_dir(src_dir: &str, target: &str) {
    let zipfile = std::fs::File::create(target).unwrap();
    let dir = WalkDir::new(src_dir);
    let _ = zip_dir(&mut dir.into_iter().filter_map(|e| e.ok()), src_dir, zipfile);
}
 
 
fn zip_dir<T>(it: &mut dyn Iterator<Item=DirEntry>, prefix: &str, writer: T) -> zip::result::ZipResult<()>
    where T: Write + Seek {
    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(zip::CompressionMethod::Stored);
 
    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        println!("prefix  {:?} ...", prefix);
        let name = path.strip_prefix(Path::new(prefix)).unwrap();
        println!("name  {:?} ...", name);
        if path.is_file() {
            println!("adding file {:?} as {:?} ...", path, name);
            // zip.start_file_from_path(name, options)?;
            zip.start_file(name.to_string_lossy(), options)?;
            let mut f = File::open(path)?;
 
            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            println!("adding dir {:?} as {:?} ...", path, name);
            // zip.add_directory_from_path(name, options)?;
            zip.add_directory(name.to_string_lossy(), options)?;
        }
    }
    zip.finish()?;
    Result::Ok(())
}
 

#[derive(Debug, Serialize, Deserialize)]
struct BackupConfig {
    name: String,
    from: String,
    dest: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    backup: Vec<BackupConfig>,
    settings: Settings,
}

#[derive(Debug, Serialize, Deserialize)]
struct Settings {
    interval: u64,
    saving_name: String,
}

fn about() {
    println!("but v0.2，文件夹备份工具。");
    println!("by @SO-TS (github.com/SO-TS)");
    println!("在 https://github.com/SO-TS/but/issues 上反馈 Bugs 或 Feature Requests。");
}

fn init_config() {
    let config = Config {
        backup: vec![BackupConfig {
            name: "test".to_string(),
            from: "path/to/folder".to_string(),
            dest: "./".to_string(),
        }],
        settings: Settings {
            interval: 300,
            saving_name: "%name%-%timestamp%".to_string(),
        },
    };

    let config_path = PathBuf::from("but.json");
    if let Err(e) = write_to_file(&config_path, &config) {
        error!("无法初始化配置文件: {}", e);
        exit(1);
    }

    println!("配置文件生成完毕，请重新执行 but。");
    exit(0);
}

fn write_to_file(path: &PathBuf, config: &Config) -> Result<(), std::io::Error> {
    let mut file = File::create(path)?;
    serde_json::to_writer_pretty(&mut file, config)?;
    Ok(())
}

fn load_config() -> Config {
    let config_path = PathBuf::from("but.json");

    if !config_path.exists() {
        init_config();
    }

    let file = File::open(config_path).expect("无法打开配置文件");
    let config: Config = serde_json::from_reader(file).unwrap_or_else(|e| {
        error!("配置文件无法加载，原因为: {}", e);
        exit(1);
    });

    config
}

fn start_listen() {
    let mut now = SystemTime::now();
    println!("开始备份监听。");

    loop {
        let config = load_config();
        std::thread::sleep(std::time::Duration::from_secs(1));

        if now.elapsed().unwrap().as_secs() > config.settings.interval as u64 {
            for (i, backup_item) in config.backup.iter().enumerate() {
                println!(
                    "[{}/{}] 正在备份 {}",
                    i + 1,
                    config.backup.len(),
                    backup_item.name
                );

                let backup_name = format!(
                    "{}-{}",
                    backup_item.name.replace("%name%", &backup_item.name),
                    SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                );

                let dest_path = format!("{}/{}.zip", backup_item.dest, backup_name);
                zip_folder(backup_item.from.as_str(), dest_path.as_str()).expect("压缩失败");

                println!("本次备份全部完成。");

                now = SystemTime::now();
            }
        }
    }
}

fn zip_folder(from: &str, dest: &str) -> Result<(), Box<dyn std::error::Error>> {
    compress_dir(from, dest);
    Ok(())
}

fn help() {
    println!(
        r#"but - 使用方法:

    -v, --version, --about 显示关于信息
    -i, -g, --init         生成配置文件

无参数启动将开始直接备份。

如果您是第一次使用，请输入 "but --init" 生成配置文件。
请对配置文件进行修改来使用 but，后续更新将添加更多功能。
不留参数将会开始文件备份。"#
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|arg| arg.as_str()) {
        None => start_listen(),
        Some("--version") | Some("-v") | Some("--about") => about(),
        Some("-i") | Some("-g") | Some("--init") => init_config(),
        _ => help(),
    };
}
