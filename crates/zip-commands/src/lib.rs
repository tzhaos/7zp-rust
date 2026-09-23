use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const CLSID: &str = "{50727386-A1B4-4CFA-B327-C510F503714D}";
pub const CLSID_VALUE: u128 = 0x50727386_a1b4_4cfa_b327_c510f503714d;
pub const SHELL_KEY: &str = r"Software\Plus7z\Shell";
pub const REQUEST_PREFIX: &str = "p7z-shell-";
pub const EXTENSIONS: &[&str] = &[
    ".001",
    ".7z",
    ".apm",
    ".apk",
    ".appx",
    ".ar",
    ".arj",
    ".bz2",
    ".bzip2",
    ".cab",
    ".chm",
    ".cpio",
    ".cramfs",
    ".deb",
    ".dmg",
    ".epub",
    ".esd",
    ".ext",
    ".ext2",
    ".ext3",
    ".ext4",
    ".fat",
    ".gpt",
    ".gz",
    ".gzip",
    ".hfs",
    ".hfsx",
    ".ihex",
    ".img",
    ".ipa",
    ".iso",
    ".jar",
    ".lha",
    ".lzh",
    ".lzma",
    ".mbr",
    ".msix",
    ".ntfs",
    ".qcow",
    ".qcow2",
    ".rar",
    ".rpm",
    ".squashfs",
    ".swm",
    ".tar",
    ".taz",
    ".tbz",
    ".tbz2",
    ".tgz",
    ".txz",
    ".udf",
    ".vdi",
    ".vhd",
    ".vhdx",
    ".vmdk",
    ".wim",
    ".xar",
    ".xz",
    ".z",
    ".zip",
    ".zst",
];

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Action {
    Open,
    OpenAs(OpenType),
    OneClickExtract,
    Extract,
    ExtractHere,
    ExtractFolder,
    Compress,
    CompressEmail,
    QuickCompress(ArchiveFormat, bool),
    Check,
    Checksum(HashMethod),
    GenerateChecksum,
    VerifyChecksum,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpenType {
    Auto,
    Parser,
    ParserEach,
    SevenZip,
    Zip,
    Cab,
    Rar,
}
impl OpenType {
    pub fn value(self) -> &'static str {
        match self {
            Self::Auto => "*",
            Self::Parser => "#",
            Self::ParserEach => "#:e",
            Self::SevenZip => "7z",
            Self::Zip => "zip",
            Self::Cab => "cab",
            Self::Rar => "rar",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchiveFormat {
    SevenZip,
    Zip,
}
impl ArchiveFormat {
    pub fn value(self) -> &'static str {
        match self {
            Self::SevenZip => "7z",
            Self::Zip => "zip",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HashMethod {
    Crc32,
    Crc64,
    Xxh64,
    Md5,
    Sha1,
    Sha256,
    Sha384,
    Sha512,
    Sha3_256,
    Blake2sp,
    All,
}
impl HashMethod {
    pub fn value(self) -> &'static str {
        match self {
            Self::Crc32 => "CRC32",
            Self::Crc64 => "CRC64",
            Self::Xxh64 => "XXH64",
            Self::Md5 => "MD5",
            Self::Sha1 => "SHA1",
            Self::Sha256 => "SHA256",
            Self::Sha384 => "SHA384",
            Self::Sha512 => "SHA512",
            Self::Sha3_256 => "SHA3-256",
            Self::Blake2sp => "BLAKE2sp",
            Self::All => "*",
        }
    }
    pub fn title(self) -> &'static str {
        match self {
            Self::Crc32 => "CRC-32",
            Self::Crc64 => "CRC-64",
            Self::Sha1 => "SHA-1",
            Self::Sha256 => "SHA-256",
            Self::Sha384 => "SHA-384",
            Self::Sha512 => "SHA-512",
            _ => self.value(),
        }
    }
}

pub const ACTIONS: &[Action] = &[
    Action::Open,
    Action::OpenAs(OpenType::Auto),
    Action::OpenAs(OpenType::Parser),
    Action::OpenAs(OpenType::ParserEach),
    Action::OpenAs(OpenType::SevenZip),
    Action::OpenAs(OpenType::Zip),
    Action::OpenAs(OpenType::Cab),
    Action::OpenAs(OpenType::Rar),
    Action::OneClickExtract,
    Action::Extract,
    Action::ExtractHere,
    Action::ExtractFolder,
    Action::Check,
    Action::Compress,
    Action::CompressEmail,
    Action::QuickCompress(ArchiveFormat::SevenZip, false),
    Action::QuickCompress(ArchiveFormat::SevenZip, true),
    Action::QuickCompress(ArchiveFormat::Zip, false),
    Action::QuickCompress(ArchiveFormat::Zip, true),
    Action::Checksum(HashMethod::Crc32),
    Action::Checksum(HashMethod::Crc64),
    Action::Checksum(HashMethod::Xxh64),
    Action::Checksum(HashMethod::Md5),
    Action::Checksum(HashMethod::Sha1),
    Action::Checksum(HashMethod::Sha256),
    Action::Checksum(HashMethod::Sha384),
    Action::Checksum(HashMethod::Sha512),
    Action::Checksum(HashMethod::Sha3_256),
    Action::Checksum(HashMethod::Blake2sp),
    Action::Checksum(HashMethod::All),
    Action::GenerateChecksum,
    Action::VerifyChecksum,
];

impl Action {
    pub fn argument(self) -> &'static str {
        match self {
            Self::Open => "--open",
            Self::OpenAs(kind) => match kind {
                OpenType::Auto => "--open-auto",
                OpenType::Parser => "--open-parser",
                OpenType::ParserEach => "--open-parser-each",
                OpenType::SevenZip => "--open-7z",
                OpenType::Zip => "--open-zip",
                OpenType::Cab => "--open-cab",
                OpenType::Rar => "--open-rar",
            },
            Self::OneClickExtract => "--one-click-extract",
            Self::Extract => "--extract",
            Self::ExtractHere => "--extract-here",
            Self::ExtractFolder => "--extract-folder",
            Self::Compress => "--compress",
            Self::CompressEmail => "--compress-email",
            Self::QuickCompress(ArchiveFormat::SevenZip, false) => "--compress-7z",
            Self::QuickCompress(ArchiveFormat::SevenZip, true) => "--compress-7z-email",
            Self::QuickCompress(ArchiveFormat::Zip, false) => "--compress-zip",
            Self::QuickCompress(ArchiveFormat::Zip, true) => "--compress-zip-email",
            Self::Check => "--check",
            Self::Checksum(method) => match method {
                HashMethod::Crc32 => "--hash-crc32",
                HashMethod::Crc64 => "--hash-crc64",
                HashMethod::Xxh64 => "--hash-xxh64",
                HashMethod::Md5 => "--hash-md5",
                HashMethod::Sha1 => "--hash-sha1",
                HashMethod::Sha256 => "--hash-sha256",
                HashMethod::Sha384 => "--hash-sha384",
                HashMethod::Sha512 => "--hash-sha512",
                HashMethod::Sha3_256 => "--hash-sha3-256",
                HashMethod::Blake2sp => "--hash-blake2sp",
                HashMethod::All => "--hash-all",
            },
            Self::GenerateChecksum => "--hash-generate",
            Self::VerifyChecksum => "--hash-verify",
        }
    }

    pub fn label_key(self) -> &'static str {
        match self {
            Self::Open => "archive-open",
            Self::OpenAs(_) => "archive-open",
            Self::OneClickExtract => "shell-one-click-extract",
            Self::Extract => "shell-extract-files",
            Self::ExtractHere => "extract-here",
            Self::ExtractFolder => "shell-extract-named",
            Self::Compress => "shell-add-archive",
            Self::CompressEmail => "shell-compress-email",
            Self::QuickCompress(_, false) => "shell-compress-named",
            Self::QuickCompress(_, true) => "shell-compress-named-email",
            Self::Check => "shell-test-archive",
            Self::Checksum(_) => "shell-checksums",
            Self::GenerateChecksum => "shell-generate-checksum",
            Self::VerifyChecksum => "shell-verify-checksum",
        }
    }

    pub fn available(self, count: usize, all_archives: bool) -> bool {
        count > 0
            && match self {
                Self::Open | Self::OpenAs(_) => count == 1 && all_archives,
                Self::Extract
                | Self::ExtractHere
                | Self::ExtractFolder
                | Self::OneClickExtract
                | Self::Check => all_archives,
                _ => true,
            }
    }
}

pub fn opens_directly(path: &Path) -> bool {
    const EXTENSIONS: &[&str] = &[
        "7z", "zip", "rar", "tar", "gz", "bz2", "xz", "wim", "iso", "cab", "001",
    ];
    path.extension().is_some_and(|extension| {
        EXTENSIONS
            .iter()
            .any(|known| extension.eq_ignore_ascii_case(known))
    })
}

// Match 26.03 Explorer's exclusion policy, including unrecognised archive extensions.
pub fn may_extract(path: &Path) -> bool {
    const EXCLUDED: &str = "3gp aac ans ape asc asm asp aspx avi awk bas bat bmp c cs cls clw cmd cpp csproj css ctl cxx def dep dlg dsp dsw eps f f77 f90 f95 fla flac frm gif h hpp hta htm html hxx ico idl inc ini inl java jpeg jpg js la lnk log mak manifest wmv mov mp3 mp4 mpe mpeg mpg m4a ofr ogg pac pas pdf php php3 php4 php5 phptml pl pm png ps py pyo ra rb rc reg rka rm rtf sed sh shn shtml sln sql srt swa tcl tex tiff tta txt vb vcproj vbs mkv wav webm wma wv xml xsd xsl xslt";
    !path.extension().is_some_and(|extension| {
        EXCLUDED
            .split_whitespace()
            .any(|known| extension.to_string_lossy().eq_ignore_ascii_case(known))
    })
}

// Naming is shared by Explorer titles and execution; inspect only the supplied selection.
pub fn archive_name(paths: &[PathBuf], first_directory: bool, hash: bool) -> String {
    let first = &paths[0];
    let mut name = if paths.len() == 1 {
        let name = first
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        if !hash && !first_directory && name.matches('.').count() == 1 && !name.starts_with('.') {
            name.split_once('.').unwrap().0.to_owned()
        } else {
            name
        }
    } else {
        let parent = first.parent().unwrap_or(Path::new(""));
        parent
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .unwrap_or_else(|| {
                parent
                    .to_string_lossy()
                    .trim_end_matches([':', '\\', '/'])
                    .to_owned()
            })
    };
    if name.is_empty() {
        name = "Archive".into();
    }
    let extensions: &[&str] = if hash {
        &["sha256"]
    } else {
        &["7z", "zip", "tar", "wim"]
    };
    let collides = |base: &str| {
        paths.iter().any(|path| {
            extensions.iter().any(|ext| {
                path.file_name().is_some_and(|file| {
                    file.to_string_lossy()
                        .eq_ignore_ascii_case(&format!("{base}.{ext}"))
                })
            })
        })
    };
    if collides(&name) {
        let base = name.clone();
        let mut number = 2;
        loop {
            name = format!("{base}_{number}");
            if !collides(&name) {
                break;
            }
            number += 1;
        }
    }
    name
}

pub fn extract_folder(path: &Path) -> String {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    let Some((stem, extension)) = name.rsplit_once('.') else {
        return format!("{name}~");
    };
    let mut stem = stem.trim_end();
    if let Some((base, inner)) = stem.rsplit_once('.') {
        let volume = extension.eq_ignore_ascii_case("001")
            && ["7z", "bz2", "gz", "rar", "zip"]
                .iter()
                .any(|known| inner.eq_ignore_ascii_case(known));
        let rar = extension.eq_ignore_ascii_case("rar")
            && ["part001", "part01", "part1"]
                .iter()
                .any(|known| inner.eq_ignore_ascii_case(known));
        if !base.is_empty() && (volume || rar) {
            stem = base.trim_end();
        }
    }
    let stem = stem.trim_end_matches([' ', '.']);
    if stem.is_empty() {
        "_".into()
    } else {
        stem.into()
    }
}

#[derive(Serialize, Deserialize)]
pub struct Request {
    pub action: Action,
    pub paths: Vec<PathBuf>,
}
