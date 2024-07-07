use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::process::exit;
use std::time::{SystemTime, UNIX_EPOCH};
use tar::Builder;
use walkdir::{DirEntry, WalkDir};
use zip::{write::FileOptions, write::ZipWriter};
use zstd::stream::Encoder;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Compression {
    Zip,
    Zstd,
}

#[derive(Debug, Serialize, Deserialize)]
struct Backup {
    name: String,
    from: String,
    dest: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    backup: Vec<Backup>,
    settings: Settings,
}

#[derive(Debug, Serialize, Deserialize)]
struct Settings {
    interval: u64,
    saving_name: String,
    compression: Compression,
}

fn about() {
    println!("but v0.3，文件夹备份工具。");
}

fn init_config() {
    let cfg = Config {
        backup: vec![Backup {
            name: "test".to_string(),
            from: "path/to/folder".to_string(),
            dest: "./".to_string(),
        }],
        settings: Settings {
            interval: 300,
            saving_name: "%name%-%timestamp%".to_string(),
            compression: Compression::Zip,
        },
    };
    write_config_file("but.json", &cfg).unwrap_or_else(|e| {
        eprintln!("无法初始化配置文件: {}", e);
        exit(1);
    });
    println!("配置文件生成完毕，请重新执行 but。");
    exit(0);
}

fn write_config_file(path: &str, cfg: &Config) -> Result<(), std::io::Error> {
    let mut file = File::create(path)?;
    serde_json::to_writer_pretty(&mut file, cfg)?;
    Ok(())
}

fn load_config() -> Result<Config, std::io::Error> {
    let config_paths = vec![
        PathBuf::from("/etc/but.json"),
        PathBuf::from(format!(
            "{}/.config/but.json",
            env::var("HOME").unwrap_or_else(|_| ".".to_string())
        )),
        PathBuf::from("but.json"),
    ];
    for path in config_paths {
        if path.exists() {
            let file = File::open(path)?;
            let cfg: Config = serde_json::from_reader(file)?;
            return Ok(cfg);
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        "配置文件不存在",
    ))
}

fn start_listen() {
    let mut now = SystemTime::now();
    println!("开始备份监听。");

    let current_config = load_config().unwrap_or_else(|e| {
        eprintln!("配置文件无法加载，原因为: {}", e);
        exit(1);
    });

    loop {
        std::thread::sleep(std::time::Duration::from_secs(1));

        if now.elapsed().unwrap().as_secs() > current_config.settings.interval {
            for (i, item) in current_config.backup.iter().enumerate() {
                let mut files = Vec::new();

                for entry in WalkDir::new(&item.from) {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    if path.is_file() {
                        files.push(path.to_string_lossy().to_string());
                    }
                }

                println!(
                    "[{}/{}] 正在备份 {}",
                    i + 1,
                    current_config.backup.len(),
                    item.name
                );

                let backup_name = format!(
                    "{}-{}",
                    item.name.replace("%name%", &item.name),
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs()
                );

                let dest_path = format!(
                    "{}/{}.{}",
                    item.dest,
                    backup_name,
                    match current_config.settings.compression {
                        Compression::Zip => "zip",
                        Compression::Zstd => "tar.zst",
                    }
                );

                compress_folder(
                    item.from.as_str(),
                    dest_path.as_str(),
                    &current_config.settings.compression,
                )
                .expect("压缩失败");

                println!("本次备份全部完成。");
            }

            now = SystemTime::now();
        }
    }
}

fn compress_folder(
    from: &str,
    dest: &str,
    compression: &Compression,
) -> Result<(), Box<dyn std::error::Error>> {
    match compression {
        Compression::Zip => {
            let zipfile = std::fs::File::create(dest).unwrap();
            let dir = WalkDir::new(from);
            zip_dir(&mut dir.into_iter().filter_map(|e| e.ok()), from, zipfile)?;
        }
        Compression::Zstd => {
            let tar_file = File::create(dest).unwrap();
            let encoder = Encoder::new(tar_file, 3).unwrap();
            let mut tar_builder = Builder::new(encoder);

            for entry in WalkDir::new(from) {
                let entry = entry.unwrap();
                let path = entry.path();
                let name = path.strip_prefix(Path::new(from)).unwrap();
                if path.is_file() {
                    tar_builder.append_path_with_name(path, name).unwrap();
                }
            }

            tar_builder.into_inner().unwrap().finish().unwrap();
        }
    }
    Ok(())
}

fn zip_dir<T>(
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: &str,
    writer: T,
) -> zip::result::ZipResult<()>
where
    T: Write + Seek,
{
    let mut zip = ZipWriter::new(writer);
    let options: FileOptions<'_, ()> =
        FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix)).unwrap();
        if path.is_file() {
            zip.start_file(name.to_string_lossy(), options)?;
            let mut f = File::open(path)?;
            f.read_to_end(&mut buffer)?;
        }
    }
    Ok(())
}

fn help() {
    println!(
        r#"but - 使用方法:

配置操作:
    -i, -g, --init         生成配置文件
    -V                     显示版本信息

如果您是第一次使用，请输入 "but --init" 生成配置文件。
请对配置文件进行修改来使用 but，后续更新将添加更多功能。
不留参数将会开始文件备份。"#
    );
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|arg| arg.as_str()) {
        None => start_listen(),
        Some("--version") | Some("-V") | Some("--about") => about(),
        Some("-i") | Some("-g") | Some("--init") => init_config(),
        _ => help(),
    };
}
