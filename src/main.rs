use clap::Parser;
use http_body_util::Full;
use hyper::body::{Body, Bytes};
use hyper::header::{HeaderValue, CONTENT_TYPE};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncReadExt;
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use walkdir::WalkDir;

/// Command line arguments for Simplex HTTP Server
#[derive(Parser, Debug)]
#[command(name = "simplex")]
struct Args {
    /// Host to bind the server to
    #[arg(short = 'a', long, default_value = "127.0.0.1")]
    host: String,

    /// Port to bind the server to
    #[arg(short, long, default_value_t = 3000)]
    port: u16,

    /// Enable logging of requested files
    #[arg(short, long, action = clap::ArgAction::SetTrue, default_value_t = false)]
    log: bool,
}

type FileMap = HashMap<String, (String, Bytes)>;

/// Serves the requested file from the in-memory file map.
///
/// # Arguments
///
/// * `req` - The incoming HTTP request.
/// * `files` - An `Arc` to a `RwLock` containing the in-memory file map.
/// * `log` - A boolean indicating whether logging is enabled.
///
/// # Returns
///
/// A `Result` containing the HTTP response.
async fn serve_file(
    req: Request<hyper::body::Incoming>,
    files: Arc<RwLock<FileMap>>,
    log: bool,
) -> Result<Response<Full<Bytes>>, Infallible> {
    let path = req.uri().path().to_string();
    let path = if path == "/" {
        "/index.html".to_string()
    } else {
        path
    };

    if log {
        println!("Request for: {:?}", path);
    }

    let files = files.read().await;

    if let Some((content_type, contents)) = files.get(&path) {
        Ok(Response::builder()
            .header(CONTENT_TYPE, HeaderValue::from_str(content_type).unwrap())
            .body(Full::new(contents.clone()))
            .unwrap())
    } else {
        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Full::new(Bytes::from("File Not Found")))
            .unwrap())
    }
}

/// Loads all files from the specified root directory into an in-memory file map.
///
/// # Arguments
///
/// * `root_dir` - The root directory to load files from.
///
/// # Returns
///
/// A `Result` containing the in-memory file map.
async fn load_files(
    root_dir: PathBuf,
) -> Result<FileMap, Box<dyn std::error::Error + Send + Sync>> {
    let mut files = FileMap::new();

    for entry in WalkDir::new(&root_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            let content_type = match path.extension().and_then(|ext| ext.to_str()) {
                Some("html") => "text/html",
                Some("css") => "text/css",
                Some("js") => "application/javascript",
                Some("png") => "image/png",
                Some("jpg") | Some("jpeg") => "image/jpeg",
                Some("gif") => "image/gif",
                Some("svg") => "image/svg+xml",
                Some("woff") => "font/woff",
                Some("woff2") => "font/woff2",
                Some("ttf") => "font/ttf",
                Some("otf") => "font/otf",
                Some("mp3") => "audio/mpeg",
                Some("wav") => "audio/wav",
                Some("ogg") => "audio/ogg",
                Some("mp4") => "video/mp4",
                Some("webm") => "video/webm",
                Some("json") => "application/json",
                Some("pdf") => "application/pdf",
                Some("ico") => "image/x-icon",
                _ => "application/octet-stream",
            };

            let mut file = fs::File::open(&path).await?;
            let mut contents = Vec::new();
            file.read_to_end(&mut contents).await?;

            let relative_path = path
                .strip_prefix(&root_dir)
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            let key = format!("/{}", relative_path.replace("\\", "/"));
            files.insert(key, (content_type.to_string(), Bytes::from(contents)));
        }
    }

    Ok(files)
}

/// Main entry point for the Simplex HTTP Server.
///
/// Parses command line arguments, loads files from the current directory,
/// and starts the HTTP server.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = Args::parse();

    // Set the root directory based on the build type
    let root_dir = {
        #[cfg(debug_assertions)]
        {
            // In debug mode, use the "html" directory if it exists
            let dev_dir = std::env::current_dir()?.join("html");
            if std::fs::metadata(&dev_dir).is_ok() {
                dev_dir
            } else {
                // Fall back to the current directory if "html" does not exist
                std::env::current_dir()?
            }
        }
        #[cfg(not(debug_assertions))]
        {
            // In release mode, use the current working directory
            std::env::current_dir()?
        }
    };

    println!("Loading files from: {:?}", root_dir);
    let files = load_files(root_dir).await?;
    let files = Arc::new(RwLock::new(files));

    println!("Server Started on {}:{}", args.host, args.port);
    println!("URL: http://{}:{}", args.host, args.port);
    let addr = SocketAddr::new(args.host.parse()?, args.port);
    let listener = TcpListener::bind(addr).await?;

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let files = Arc::clone(&files);
        let log = args.log;

        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(
                    io,
                    service_fn(move |req| serve_file(req, Arc::clone(&files), log)),
                )
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_load_files() {
        let root_dir = PathBuf::from("./html");
        let files = load_files(root_dir).await.unwrap();

        assert!(files.contains_key("/index.html"));
        assert!(files.contains_key("/css/style.css"));
        assert_eq!(files["/index.html"].0, "text/html");
        assert_eq!(files["/css/style.css"].0, "text/css");
    }
}
