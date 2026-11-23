use super::nfo::ParsedNfo;
use std::path::Path;

/// Parse TinyMediaManager-style folder names when no NFO is present.
/// Example: `Good Sam (2019) {imdb-tt9032888} [1080p]`
pub fn parsed_from_folder_name(folder: &Path) -> ParsedNfo {
    let name = folder
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();

    let mut parsed = ParsedNfo::default();

    if let Some((title, year)) = split_title_year(name) {
        parsed.title = Some(title);
        parsed.year = year;
    } else if !name.is_empty() {
        parsed.title = Some(name.to_string());
    }

    if let Some(imdb) = extract_imdb_token(name) {
        parsed.imdb_id = Some(normalize_imdb_id(&imdb));
    }

    if let Some(name) = folder.file_name().and_then(|value| value.to_str()) {
        if let Some(height) = resolution_from_text(name) {
            parsed.resolution_height = Some(height);
            parsed.resolution_width = Some(default_width_for_height(height));
        }
    }

    parsed
}

/// Fill resolution from the video file name when still unknown after NFO / folder parsing.
pub fn apply_resolution_from_video_name(parsed: &mut ParsedNfo, video: &Path) {
    if parsed.resolution_height.is_some() {
        return;
    }
    let name = video
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if let Some(height) = resolution_from_text(name) {
        parsed.resolution_height = Some(height);
        parsed.resolution_width = Some(default_width_for_height(height));
    }
}

fn split_title_year(name: &str) -> Option<(String, Option<i32>)> {
    let open = name.rfind('(')?;
    let rest = &name[open + 1..];
    let year_end = rest.find(')')?;
    let year_str = rest.get(..year_end)?.trim();
    let year = year_str.parse::<i32>().ok();
    let title = name.get(..open)?.trim().to_string();
    if title.is_empty() {
        return None;
    }
    Some((title, year))
}

fn extract_imdb_token(name: &str) -> Option<String> {
    let lower = name.to_ascii_lowercase();
    let marker = "{imdb-";
    let start = lower.find(marker)? + marker.len();
    let tail = name.get(start..)?;
    let end = tail.find('}')?;
    let token = tail.get(..end)?.trim();
    if token.is_empty() {
        None
    } else {
        Some(token.to_string())
    }
}

fn normalize_imdb_id(token: &str) -> String {
    let lower = token.to_ascii_lowercase();
    if lower.starts_with("tt") {
        lower
    } else if lower.starts_with("imdb-tt") {
        lower.strip_prefix("imdb-").unwrap_or(&lower).to_string()
    } else if token.chars().all(|c| c.is_ascii_digit()) {
        format!("tt{token}")
    } else {
        lower
    }
}

fn resolution_from_text(text: &str) -> Option<i32> {
    let lower = text.to_ascii_lowercase();
    for (tag, height) in [
        ("[2160p]", 2160),
        ("[4k]", 2160),
        ("[1080p]", 1080),
        ("[720p]", 720),
        ("[576p]", 576),
        ("[480p]", 480),
    ] {
        if lower.contains(tag) {
            return Some(height);
        }
    }
    for (token, height) in [
        ("2160p", 2160),
        ("1080p", 1080),
        ("720p", 720),
        ("576p", 576),
        ("480p", 480),
        ("4k", 2160),
    ] {
        if contains_resolution_token(&lower, token) {
            return Some(height);
        }
    }
    None
}

fn contains_resolution_token(lower: &str, token: &str) -> bool {
    if token.is_empty() {
        return false;
    }
    let token = token.to_ascii_lowercase();
    let bytes = lower.as_bytes();
    let token_bytes = token.as_bytes();
    let mut start = 0;
    while let Some(rel) = lower[start..].find(&token) {
        let idx = start + rel;
        let before_ok = idx == 0 || !bytes[idx - 1].is_ascii_alphanumeric();
        let after_idx = idx + token_bytes.len();
        let after_ok = after_idx >= bytes.len() || !bytes[after_idx].is_ascii_alphanumeric();
        if before_ok && after_ok {
            return true;
        }
        start = idx + 1;
    }
    false
}

fn default_width_for_height(height: i32) -> i32 {
    match height {
        2160 => 3840,
        1080 => 1920,
        720 => 1280,
        576 => 1024,
        480 => 854,
        _ => (height as f64 * 16.0 / 9.0).round() as i32,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn parses_tmm_style_folder() {
        let path = Path::new(r"F:\xMovies\1080p\Good Sam (2019) {imdb-tt9032888} [1080p]");
        let parsed = parsed_from_folder_name(path);
        assert_eq!(parsed.title.as_deref(), Some("Good Sam"));
        assert_eq!(parsed.year, Some(2019));
        assert_eq!(parsed.imdb_id.as_deref(), Some("tt9032888"));
        assert_eq!(parsed.resolution_height, Some(1080));
        assert_eq!(parsed.resolution_width, Some(1920));
    }

    #[test]
    fn parses_year_only_title() {
        let path = Path::new(r"DEADPOOL (2016) {IMDB-TT1431045} [2160P]");
        let parsed = parsed_from_folder_name(path);
        assert_eq!(parsed.title.as_deref(), Some("DEADPOOL"));
        assert_eq!(parsed.year, Some(2016));
        assert_eq!(parsed.imdb_id.as_deref(), Some("tt1431045"));
        assert_eq!(parsed.resolution_height, Some(2160));
    }

    #[test]
    fn does_not_infer_resolution_from_parent_folder() {
        let path = Path::new(r"F:\xMovies\1080p\Inception (2010)");
        let parsed = parsed_from_folder_name(path);
        assert!(parsed.resolution_height.is_none());
    }

    #[test]
    fn infers_resolution_from_video_file_name() {
        let mut parsed = ParsedNfo::default();
        apply_resolution_from_video_name(
            &mut parsed,
            Path::new(r"F:\movies\Title\Title.2010.1080p.BluRay.x264.mkv"),
        );
        assert_eq!(parsed.resolution_height, Some(1080));
    }
}