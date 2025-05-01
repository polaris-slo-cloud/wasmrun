use serde::Deserialize;
use serde_json::{json, Value};

use std::fs::File;
use std::io::{Read, Seek, Write};
use std::iter::Iterator;
use std::path::Path;
use zip::result::ZipError;
use zip::write::FileOptions;

use walkdir::{DirEntry, WalkDir};

use shared_lib::storage_utils::{save_to_local};

#[derive(Deserialize)]
enum CompressionType {
    Stored,
    Deflated,
//    Bzip2,
//    Zstd,
}

#[derive(Deserialize)]
struct Json {
    storage_type: String,
    input_path: String,
    output_path: String,
    compression_type: CompressionType,
    //#[serde(default)]
    //bucket: Option<String>,
}

const METHOD_STORED: Option<zip::CompressionMethod> = Some(zip::CompressionMethod::Stored);
const METHOD_DEFLATED: Option<zip::CompressionMethod> = Some(zip::CompressionMethod::Deflated);
//const METHOD_BZIP2: Option<zip::CompressionMethod> = Some(zip::CompressionMethod::Bzip2);
//const METHOD_ZSTD: Option<zip::CompressionMethod> = Some(zip::CompressionMethod::Zstd);

pub async fn handle_json(json: Value) -> Result<String, String> {
    let result: Result<Json, _> = serde_json::from_value(json);

    match result {
        Ok(valid_json) => {

            match valid_json.storage_type.as_str() {
                //"s3" => {
                //    get_dir_s3(&valid_json.bucket.unwrap(), &valid_json.input_path).await?;
                //}
                "local" => {
                    compress(
                        &valid_json.input_path,
                        &valid_json.output_path,
                        valid_json.compression_type,
                    )?;
                }
                _ => {
                    return Err("Unsupported storage type".to_string());
                }
            };


            //let save_result = match valid_json.storage_type.as_str() {
            //    //"s3" => save_to_s3(&valid_json.bucket.unwrap(), &valid_json.output_path, &image_data).await?,
            //    "local" => save_to_local(&valid_json.output_path, &image_data).await?,
            //    _ => {
            //        return Err("Unsupported storage type".to_string());
            //    }
            //};

            let response = json!({
                "status": "success",
                "runtime": "wasm",
                "data": {
                    "save_path" : valid_json.output_path
                }
            });

            Ok(response.to_string())
        }
        Err(e) => {
            let response = json!({
                "status": "error",
                "message": e.to_string()
            });
            Ok(response.to_string())
        }
    }
}

fn compress(src_dir: &str, dst_file: &str, comp_type: CompressionType) -> Result<String, String> {
    let method = match comp_type {
        CompressionType::Stored => Some(METHOD_STORED),
        CompressionType::Deflated => Some(METHOD_DEFLATED),
//        CompressionType::Bzip2 => Some(METHOD_BZIP2),
//        CompressionType::Zstd => Some(METHOD_ZSTD),
    };

    if let Some(method) = method {
        match doit(src_dir, dst_file, method.unwrap()) {
            Ok(_) => {
                let method_name = format!("{:?}", comp_type.to_string());
                Ok(format!(
                    "Successfully compressed with method: {}",
                    method_name
                ))
            }
            Err(e) => Err(format!("Error during compression: {:?}", e)),
        }
    } else {
        Err("Unsupported compression method".to_string())
    }
}

fn zip_dir<T>(
    it: &mut dyn Iterator<Item = DirEntry>,
    prefix: &str,
    writer: T,
    method: zip::CompressionMethod,
) -> zip::result::ZipResult<()>
where
    T: Write + Seek,
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();
        let name = path.strip_prefix(Path::new(prefix)).unwrap();

        // Write file or directory explicitly
        // Some unzip tools unzip files with directory paths correctly, some do not!
        if path.is_file() {
            println!("adding file {:?} as {:?} ...", path, name);
            #[allow(deprecated)]
            zip.start_file_from_path(name, options)?;
            let mut f = File::open(path)?;

            f.read_to_end(&mut buffer)?;
            zip.write_all(&*buffer)?;
            buffer.clear();
        } else if !name.as_os_str().is_empty() {
            // Only if not root! Avoids path spec / warning
            // and mapname conversion failed error on unzip
            println!("adding dir {:?} as {:?} ...", path, name);
            #[allow(deprecated)]
            zip.add_directory_from_path(name, options)?;
        }
    }
    zip.finish()?;
    Result::Ok(())
}

fn doit(
    src_dir: &str,
    dst_file: &str,
    method: zip::CompressionMethod,
) -> zip::result::ZipResult<()> {
    if !Path::new(src_dir).is_dir() {
        return Err(ZipError::FileNotFound);
    }

    let path = Path::new(dst_file);
    let file = File::create(&path).unwrap();

    let walkdir = WalkDir::new(src_dir);
    let it = walkdir.into_iter();

    zip_dir(&mut it.filter_map(|e| e.ok()), src_dir, file, method)?;

    Ok(())
}

impl ToString for CompressionType {
    fn to_string(&self) -> String {
        match self {
            CompressionType::Stored => "stored".to_string(),
            CompressionType::Deflated => "deflated".to_string(),
//            CompressionType::Bzip2 => "bzip2".to_string(),
//            CompressionType::Zstd => "zstd".to_string(),
        }
    }
}