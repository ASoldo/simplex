![image](https://github.com/ASoldo/simplex/assets/1175537/c6949858-3140-478e-bfee-73ce3783d391)

# Simplex

A simple HTTP server written in Rust using Hyper and Clap. This server serves static files from the current working directory.

## Features

- Serve static files including HTML, CSS, JavaScript, images, fonts, audio, video, JSON, PDFs, and ICO files.
- Configurable port via command-line arguments.

## Installation

1. **Build the project in release mode:**

   ```sh
   cargo build --release
   ```

2. **Move the binary to `/usr/bin` or a subdirectory:**

   ```sh
   sudo mv target/release/simplex /usr/bin/
   ```

## Usage

Run the server from any directory to serve files from that directory:

```sh
cd /path/to/your/static/files
simplex
```

You can also specify the port using the --port flag:

`simplex --port 8080`

## Command-line Arguments

`--help, -h`: Show help options
`--port, -p`: Port to bind the server to (default is 3000): --port 3001, -p 3003, etc.
`--log, -l`: Print logs if set

## Example

Navigate to your static files directory:

`cd /path/to/your/static/files`
Start the server:

`simplex`
Access your site in a browser:

Open `http://127.0.0.1:3000` in your web browser.

## File Types Supported

The server supports the following file types with appropriate MIME types:

`HTML (.html)`
`CSS (.css)`
`JavaScript (.js)`
`Images (.png, .jpg, .jpeg, .gif, .svg, .ico)`
`Fonts (.woff, .woff2, .ttf, .otf)`
`Audio (.mp3, .wav, .ogg)`
`Video (.mp4, .webm)`
`JSON (.json)`
`PDF (.pdf)`
