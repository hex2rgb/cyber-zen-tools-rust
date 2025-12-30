use colored::*;
use image::ImageFormat;
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};

pub fn run_compress(src: String, dist: Option<String>, rate: f64) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "开始压缩图片...".green());
    println!("{} {}", "源路径:".cyan(), src);
    println!("{} {}", "目标路径:".cyan(), dist.as_ref().map(|s| s.as_str()).unwrap_or("未指定"));
    println!("{} {:.2}", "压缩比率:".cyan(), rate);

    if rate < 0.1 || rate > 1.0 {
        return Err("压缩比率必须在 0.1 到 1.0 之间".into());
    }

    let src_abs = fs::canonicalize(&src)?;

    if !src_abs.exists() {
        return Err(format!("源路径不存在: {}", src_abs.display()).into());
    }

    let dist_abs = if let Some(d) = dist {
        fs::canonicalize(&d).unwrap_or_else(|_| PathBuf::from(&d))
    } else {
        let timestamp = get_timestamp();
        PathBuf::from(format!("compressed_{}", timestamp))
    };

    let dist_with_timestamp = if contains_timestamp(&dist_abs) {
        dist_abs.clone()
    } else {
        let timestamp = get_timestamp();
        add_timestamp_to_path(&dist_abs, &timestamp)
    };

    if let Some(parent) = dist_with_timestamp.parent() {
        fs::create_dir_all(parent)?;
    }

    let metadata = fs::metadata(&src_abs)?;
    if metadata.is_dir() {
        compress_directory(&src_abs, &dist_with_timestamp, rate)
    } else {
        let original_ext = src_abs.extension().and_then(|s| s.to_str()).unwrap_or("");
        let original_name = src_abs.file_stem().and_then(|s| s.to_str()).unwrap_or("file");
        
        let final_dist = if dist_abs == dist_with_timestamp && !dist_with_timestamp.extension().is_some() {
            let timestamp = get_timestamp();
            dist_with_timestamp.join(format!("{}_{}.{}", original_name, timestamp, original_ext))
        } else {
            dist_with_timestamp
        };

        if let Some(parent) = final_dist.parent() {
            fs::create_dir_all(parent)?;
        }

        compress_file(&src_abs, &final_dist, rate)
    }
}

fn get_timestamp() -> String {
    let datetime = chrono::Local::now();
    datetime.format("%Y%m%d_%H%M%S").to_string()
}

fn add_timestamp_to_path(path: &Path, timestamp: &str) -> PathBuf {
    let dir = path.parent().unwrap_or(Path::new("."));
    let base = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        let name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        dir.join(format!("{}_{}.{}", name, timestamp, ext))
    } else {
        dir.join(format!("{}_{}", base, timestamp))
    }
}

fn contains_timestamp(path: &Path) -> bool {
    let re = Regex::new(r"_\d{8}_\d{6}$").unwrap();
    if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
        re.is_match(name)
    } else {
        false
    }
}

fn compress_directory(src_dir: &Path, dist_dir: &Path, rate: f64) -> Result<(), Box<dyn std::error::Error>> {
    println!("{} {}", "压缩目录:".yellow(), src_dir.display());
    
    fs::create_dir_all(dist_dir)?;

    compress_directory_recursive(src_dir, dist_dir, src_dir, rate)?;

    println!("{} {}", "✓ 目录压缩完成:".green(), dist_dir.display());
    Ok(())
}

fn compress_directory_recursive(src_dir: &Path, dist_dir: &Path, current_dir: &Path, rate: f64) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            compress_directory_recursive(src_dir, dist_dir, &path, rate)?;
            continue;
        }

        if !is_image_file(&path) {
            continue;
        }

        let rel_path = path.strip_prefix(src_dir)?;
        let dist_path = dist_dir.join(rel_path);
        
        if let Some(parent) = dist_path.parent() {
            fs::create_dir_all(parent)?;
        }

        println!("{} {}", "压缩:".cyan(), rel_path.display());
        if let Err(e) = compress_image_file(&path, &dist_path, rate) {
            println!("{} {} - {}", "压缩失败:".red(), rel_path.display(), e);
        }
    }

    Ok(())
}

fn compress_file(src_file: &Path, dist_file: &Path, rate: f64) -> Result<(), Box<dyn std::error::Error>> {
    println!("{} {}", "压缩文件:".yellow(), src_file.file_name().unwrap_or_default().to_string_lossy());
    
    if !is_image_file(src_file) {
        return Err(format!("不支持的文件格式: {:?}", src_file.extension()).into());
    }

    compress_image_file(src_file, dist_file, rate)?;
    println!("{} {}", "✓ 文件压缩完成:".green(), dist_file.display());
    Ok(())
}

fn is_image_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        let ext_lower = ext.to_lowercase();
        matches!(ext_lower.as_str(), "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp")
    } else {
        false
    }
}

fn compress_image_file(src_file: &Path, dist_file: &Path, rate: f64) -> Result<(), Box<dyn std::error::Error>> {
    let src_data = fs::read(src_file)?;
    
    let img = match image::load_from_memory(&src_data) {
        Ok(img) => img,
        Err(_) => {
            println!("{} {}，直接复制文件", "⚠️  无法解码图片:".yellow(), src_file.file_name().unwrap_or_default().to_string_lossy());
            fs::write(dist_file, &src_data)?;
            println!("{} {}", "✓ 文件复制完成:".green(), src_file.file_name().unwrap_or_default().to_string_lossy());
            println!("{} {} bytes", "  文件大小:".cyan(), src_data.len());
            return Ok(());
        }
    };

    let original_width = img.width() as i32;
    let original_height = img.height() as i32;

    let new_width = (original_width as f64 * rate).max(50.0) as u32;
    let new_height = (original_height as f64 * rate).max(50.0) as u32;

    let resized_img = img.resize(new_width, new_height, image::imageops::FilterType::Nearest);

    let format = image::ImageFormat::from_path(dist_file).unwrap_or(ImageFormat::Png);
    
    match format {
        ImageFormat::Jpeg => {
            resized_img.save_with_format(dist_file, ImageFormat::Jpeg)?;
        }
        ImageFormat::Png => {
            resized_img.save_with_format(dist_file, ImageFormat::Png)?;
        }
        ImageFormat::Gif => {
            resized_img.save_with_format(dist_file, ImageFormat::Gif)?;
        }
        _ => {
            println!("{} {:?}，直接复制文件", "⚠️  不支持的格式:".yellow(), format);
            fs::write(dist_file, &src_data)?;
            return Ok(());
        }
    }

    let dist_info = fs::metadata(dist_file)?;
    let original_size = src_data.len();
    let compressed_size = dist_info.len();
    let compression_ratio = (compressed_size as f64 / original_size as f64) * 100.0;

    println!("{} {}", "✓ 压缩完成:".green(), src_file.file_name().unwrap_or_default().to_string_lossy());
    println!("{} {}x{}", "  原始尺寸:".cyan(), original_width, original_height);
    println!("{} {}x{}", "  压缩尺寸:".cyan(), new_width, new_height);
    println!("{} {} bytes", "  原始大小:".cyan(), original_size);
    println!("{} {} bytes", "  压缩大小:".cyan(), compressed_size);
    println!("{} {:.2}%", "  压缩比率:".cyan(), compression_ratio);

    Ok(())
}

