use std::fs;
use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::thread;

pub fn run_server(dir: Option<String>, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let server_dir = dir.unwrap_or_else(|| "./".to_string());
    
    if !Path::new(&server_dir).exists() {
        eprintln!("❌ 错误: 目录 '{}' 不存在", server_dir);
        std::process::exit(1);
    }
    
    let abs_path = fs::canonicalize(&server_dir)?;
    
    if port < 1 || port > 65535 {
        eprintln!("❌ 错误: 端口号必须在 1-65535 范围内");
        std::process::exit(1);
    }
    
    if is_port_in_use(port) {
        eprintln!("❌ 错误: 端口 {} 已被占用", port);
        std::process::exit(1);
    }
    
    println!("🚀 启动静态文件服务器...");
    println!("📁 服务目录: {}", abs_path.display());
    println!("🌐 服务地址: http://localhost:{}", port);
    println!("📋 按 Ctrl+C 停止服务器\n");
    
    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr)?;
    println!("✅ 服务器已启动，监听端口 {}", port);
    
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let root_dir = abs_path.clone();
                thread::spawn(move || {
                    handle_client(stream, root_dir);
                });
            }
            Err(e) => {
                eprintln!("连接错误: {}", e);
            }
        }
    }
    
    Ok(())
}

fn is_port_in_use(port: u16) -> bool {
    TcpListener::bind(format!("127.0.0.1:{}", port)).is_err()
}

fn handle_client(mut stream: TcpStream, root_dir: PathBuf) {
    let mut buffer = [0; 1024];
    if let Ok(size) = stream.read(&mut buffer) {
        let request = String::from_utf8_lossy(&buffer[..size]);
        let lines: Vec<&str> = request.lines().collect();
        
        if let Some(first_line) = lines.first() {
            let parts: Vec<&str> = first_line.split_whitespace().collect();
            if parts.len() >= 2 {
                let method = parts[0];
                let path = parts[1];
                
                log_request(method, path);
                
                let file_path = if path == "/" {
                    root_dir.join("index.html")
                } else {
                    root_dir.join(&path[1..])
                };
                
                if file_path.starts_with(&root_dir) && file_path.exists() {
                    if file_path.is_file() {
                        if let Ok(content) = fs::read(&file_path) {
                            let response = format!(
                                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n",
                                content.len()
                            );
                            let _ = stream.write_all(response.as_bytes());
                            let _ = stream.write_all(&content);
                        }
                    } else {
                        send_404(&mut stream);
                    }
                } else {
                    send_404(&mut stream);
                }
            }
        }
    }
}

fn send_404(stream: &mut TcpStream) {
    let response = "HTTP/1.1 404 Not Found\r\n\r\n";
    let _ = stream.write_all(response.as_bytes());
}

fn log_request(method: &str, path: &str) {
    let method_icon = match method {
        "GET" => "🔍 GET",
        "POST" => "📝 POST",
        "PUT" => "✏️  PUT",
        "DELETE" => "🗑️  DELETE",
        _ => &format!("❓ {}", method),
    };
    
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let time_str = format!("{:02}:{:02}:{:02}", 
        (now / 3600) % 24, 
        (now / 60) % 60, 
        now % 60);
    
    println!("[{}] {} {} - 200", time_str, method_icon, path);
}

