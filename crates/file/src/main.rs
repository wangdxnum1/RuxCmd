mod cli;

use clap::Parser;
use cli::Cli;
use std::io::Read;

struct FileType {
    name: &'static str,
    mime: &'static str,
}

fn detect_file_type(bytes: &[u8]) -> Option<FileType> {
    let magic_rules = vec![
        (
            b"\x50\x4B\x03\x04" as &[u8],
            FileType {
                name: "ZIP archive",
                mime: "application/zip",
            },
        ),
        (
            b"\x50\x4B\x05\x06" as &[u8],
            FileType {
                name: "ZIP empty archive",
                mime: "application/zip",
            },
        ),
        (
            b"\x50\x4B\x07\x08" as &[u8],
            FileType {
                name: "ZIP spanned archive",
                mime: "application/zip",
            },
        ),
        (
            b"\x52\x61\x72\x21\x1A\x07\x00" as &[u8],
            FileType {
                name: "RAR archive",
                mime: "application/vnd.rar",
            },
        ),
        (
            b"\x37\x7A\xBC\xAF\x27\x1C" as &[u8],
            FileType {
                name: "7-Zip archive",
                mime: "application/x-7z-compressed",
            },
        ),
        (
            b"\x78\x01" as &[u8],
            FileType {
                name: "GZIP compressed",
                mime: "application/gzip",
            },
        ),
        (
            b"\x78\x9C" as &[u8],
            FileType {
                name: "GZIP compressed",
                mime: "application/gzip",
            },
        ),
        (
            b"\x78\xDA" as &[u8],
            FileType {
                name: "GZIP compressed",
                mime: "application/gzip",
            },
        ),
        (
            b"\x42\x5A\x68" as &[u8],
            FileType {
                name: "BZIP2 compressed",
                mime: "application/x-bzip2",
            },
        ),
        (
            b"\x58\x49\x42\x4B" as &[u8],
            FileType {
                name: "XZ compressed",
                mime: "application/x-xz",
            },
        ),
        (
            b"\x4C\x5A\x49\x50" as &[u8],
            FileType {
                name: "LZIP compressed",
                mime: "application/x-lzip",
            },
        ),
        (
            b"\x53\x51\x4C\x69\x74\x65\x20\x66\x6F\x72\x6D\x61\x74\x20\x33\x00" as &[u8],
            FileType {
                name: "SQLite database",
                mime: "application/vnd.sqlite3",
            },
        ),
        (
            b"\x89\x50\x4E\x47\x0D\x0A\x1A\x0A" as &[u8],
            FileType {
                name: "PNG image",
                mime: "image/png",
            },
        ),
        (
            b"\xFF\xD8\xFF" as &[u8],
            FileType {
                name: "JPEG image",
                mime: "image/jpeg",
            },
        ),
        (
            b"\x47\x49\x46\x38" as &[u8],
            FileType {
                name: "GIF image",
                mime: "image/gif",
            },
        ),
        (
            b"\x42\x4D" as &[u8],
            FileType {
                name: "BMP image",
                mime: "image/bmp",
            },
        ),
        (
            b"\x49\x49\x2A\x00" as &[u8],
            FileType {
                name: "TIFF image (little-endian)",
                mime: "image/tiff",
            },
        ),
        (
            b"\x4D\x4D\x00\x2A" as &[u8],
            FileType {
                name: "TIFF image (big-endian)",
                mime: "image/tiff",
            },
        ),
        (
            b"\x57\x45\x42\x50" as &[u8],
            FileType {
                name: "WebP image",
                mime: "image/webp",
            },
        ),
        (
            b"\x00\x00\x00\x0C\x6A\x70\x65\x67" as &[u8],
            FileType {
                name: "JPEG 2000",
                mime: "image/jp2",
            },
        ),
        (
            b"\x6A\x50\x20\x20" as &[u8],
            FileType {
                name: "JPEG 2000",
                mime: "image/jp2",
            },
        ),
        (
            b"\x52\x49\x46\x46\x00\x00\x00\x00\x57\x41\x56\x45" as &[u8],
            FileType {
                name: "WAV audio",
                mime: "audio/wav",
            },
        ),
        (
            b"\xFF\xFB" as &[u8],
            FileType {
                name: "MP3 audio",
                mime: "audio/mpeg",
            },
        ),
        (
            b"\xFF\xF3" as &[u8],
            FileType {
                name: "MP3 audio",
                mime: "audio/mpeg",
            },
        ),
        (
            b"\xFF\xF2" as &[u8],
            FileType {
                name: "MP3 audio",
                mime: "audio/mpeg",
            },
        ),
        (
            b"\x49\x44\x33" as &[u8],
            FileType {
                name: "MP3 ID3v2",
                mime: "audio/mpeg",
            },
        ),
        (
            b"\x4F\x67\x67\x53" as &[u8],
            FileType {
                name: "OGG Vorbis",
                mime: "audio/ogg",
            },
        ),
        (
            b"\x46\x4C\x41\x43" as &[u8],
            FileType {
                name: "FLAC audio",
                mime: "audio/flac",
            },
        ),
        (
            b"\x1A\x45\xDF\xA3" as &[u8],
            FileType {
                name: "Matroska video",
                mime: "video/x-matroska",
            },
        ),
        (
            b"\x00\x00\x00\x14\x66\x74\x79\x70" as &[u8],
            FileType {
                name: "MP4 video",
                mime: "video/mp4",
            },
        ),
        (
            b"\x00\x00\x00\x18\x66\x74\x79\x70\x6D\x70\x34\x32" as &[u8],
            FileType {
                name: "MP4 video",
                mime: "video/mp4",
            },
        ),
        (
            b"\x00\x00\x00\x1C\x66\x74\x79\x70" as &[u8],
            FileType {
                name: "MP4 video",
                mime: "video/mp4",
            },
        ),
        (
            b"\x66\x74\x79\x70\x69\x73\x6F\x6D" as &[u8],
            FileType {
                name: "MP4 video",
                mime: "video/mp4",
            },
        ),
        (
            b"\x00\x00\x01\xBA" as &[u8],
            FileType {
                name: "MPEG video",
                mime: "video/mpeg",
            },
        ),
        (
            b"\x00\x00\x01\xB3" as &[u8],
            FileType {
                name: "MPEG video",
                mime: "video/mpeg",
            },
        ),
        (
            b"\x4D\x5A" as &[u8],
            FileType {
                name: "Windows PE executable",
                mime: "application/vnd.microsoft.portable-executable",
            },
        ),
        (
            b"\x7F\x45\x4C\x46" as &[u8],
            FileType {
                name: "ELF executable",
                mime: "application/x-executable",
            },
        ),
        (
            b"\xCF\xFA\xED\xFE" as &[u8],
            FileType {
                name: "Mach-O executable",
                mime: "application/x-mach-binary",
            },
        ),
        (
            b"\x23\x21" as &[u8],
            FileType {
                name: "Shell script",
                mime: "text/x-shellscript",
            },
        ),
        (
            b"\x2F\x2A\x2A\x20" as &[u8],
            FileType {
                name: "Java source",
                mime: "text/x-java-source",
            },
        ),
        (
            b"\x3C\x25\x44\x4F\x43\x54\x59\x50" as &[u8],
            FileType {
                name: "PHP script",
                mime: "application/x-httpd-php",
            },
        ),
        (
            b"\x3C\x21\x44\x4F\x43\x54\x59\x50" as &[u8],
            FileType {
                name: "HTML document",
                mime: "text/html",
            },
        ),
        (
            b"\x3C\x3F\x78\x6D\x6C" as &[u8],
            FileType {
                name: "XML document",
                mime: "text/xml",
            },
        ),
        (
            b"\xFE\xFF" as &[u8],
            FileType {
                name: "UTF-16BE text",
                mime: "text/plain; charset=utf-16be",
            },
        ),
        (
            b"\xFF\xFE" as &[u8],
            FileType {
                name: "UTF-16LE text",
                mime: "text/plain; charset=utf-16le",
            },
        ),
        (
            b"\xEF\xBB\xBF" as &[u8],
            FileType {
                name: "UTF-8 text",
                mime: "text/plain; charset=utf-8",
            },
        ),
        (
            b"\x4D\x53\x43\x46" as &[u8],
            FileType {
                name: "Windows cabinet",
                mime: "application/vnd.ms-cab-compressed",
            },
        ),
        (
            b"\x43\x57\x53" as &[u8],
            FileType {
                name: "SWF Flash",
                mime: "application/x-shockwave-flash",
            },
        ),
        (
            b"\x50\x4B\x30\x30" as &[u8],
            FileType {
                name: "OpenDocument",
                mime: "application/vnd.oasis.opendocument.text",
            },
        ),
        (
            b"\x52\x65\x6E\x64\x65\x72\x50\x44\x46" as &[u8],
            FileType {
                name: "PDF document",
                mime: "application/pdf",
            },
        ),
        (
            b"\x25\x50\x44\x46\x2D" as &[u8],
            FileType {
                name: "PDF document",
                mime: "application/pdf",
            },
        ),
        (
            b"\x52\x49\x46\x46" as &[u8],
            FileType {
                name: "RIFF file",
                mime: "application/x-riff",
            },
        ),
    ];

    for (magic, ftype) in magic_rules {
        if bytes.len() >= magic.len() && &bytes[..magic.len()] == magic {
            return Some(ftype);
        }
    }

    None
}

fn read_file_header(path: &str) -> std::io::Result<Vec<u8>> {
    let mut file = std::fs::File::open(path)?;
    let mut buffer = vec![0u8; 256];
    let bytes_read = file.read(&mut buffer)?;
    buffer.truncate(bytes_read);
    Ok(buffer)
}

fn main() {
    let cli = Cli::parse();

    if cli.version {
        println!("file 0.1.0");
        return;
    }

    if cli.files.is_empty() {
        eprintln!("Usage: file [OPTIONS] <FILES>...");
        std::process::exit(1);
    }

    for file_path in cli.files {
        match read_file_header(&file_path) {
            Ok(bytes) => {
                if bytes.is_empty() {
                    println!("{}: empty", file_path);
                } else {
                    let file_type = detect_file_type(&bytes);
                    match file_type {
                        Some(ftype) => {
                            if cli.mime {
                                println!("{}: {}", file_path, ftype.mime);
                            } else {
                                println!("{}: {}", file_path, ftype.name);
                            }
                        }
                        None => {
                            println!("{}: data", file_path);
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "{}: cannot open `{}' ({})",
                    std::env::args().next().unwrap(),
                    file_path,
                    e
                );
                std::process::exit(1);
            }
        }
    }
}
