use sevenzip_core::i18n::tr;

pub fn size_text(size: u64) -> String {
    if size >= 1_048_576 {
        format!("{:.2} MB", size as f64 / 1_048_576.)
    } else if size >= 1024 {
        format!("{:.1} KB", size as f64 / 1024.)
    } else {
        format!("{size} B")
    }
}

pub fn file_kind(name: &str, directory: bool) -> &'static str {
    if directory {
        return tr("type-folder");
    }
    match name
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_lowercase()
        .as_str()
    {
        "jpg" => tr("type-jpeg"),
        "png" => tr("type-png"),
        "md" => tr("type-markdown"),
        "txt" => tr("type-text"),
        "json" => tr("type-json"),
        "csv" => tr("type-csv"),
        "zip" => tr("type-zip"),
        "7z" => tr("type-7z"),
        _ => tr("type-file"),
    }
}
