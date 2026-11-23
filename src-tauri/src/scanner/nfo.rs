use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs;
use std::path::Path;

#[derive(Debug, Default, Clone)]
pub struct ParsedNfo {
    pub title: Option<String>,
    pub year: Option<i32>,
    pub imdb_id: Option<String>,
    pub tmdb_id: Option<String>,
    pub resolution_width: Option<i32>,
    pub resolution_height: Option<i32>,
    pub video_codec: Option<String>,
    pub hdr: bool,
}

pub fn parse_movie_nfo(path: &Path) -> Result<ParsedNfo, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {e}", path.display()))?;
    parse_nfo_content(&content)
}

pub fn parse_nfo_content(content: &str) -> Result<ParsedNfo, String> {
    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);

    let mut parsed = ParsedNfo::default();
    let mut buf = Vec::new();

    let mut in_video = false;
    let mut current_uniqueid_type: Option<String> = None;
    let mut current_element: Option<String> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(tag)) => {
                let name = String::from_utf8_lossy(tag.name().as_ref()).to_string();
                if name == "video" {
                    in_video = true;
                }
                if name == "uniqueid" {
                    current_uniqueid_type = tag
                        .attributes()
                        .find(|a| {
                            a.as_ref()
                                .map(|attr| attr.key.as_ref() == b"type")
                                .unwrap_or(false)
                        })
                        .and_then(|a| a.ok())
                        .and_then(|a| {
                            String::from_utf8(a.value.to_vec()).ok()
                        });
                }
                current_element = Some(name);
            }
            Ok(Event::Text(text)) => {
                let value = text
                    .decode()
                    .map_err(|e| format!("XML text error: {e}"))?;
                let value = value.trim();
                if value.is_empty() {
                    continue;
                }

                match current_element.as_deref() {
                    Some("title") => parsed.title = Some(value.to_string()),
                    Some("year") => parsed.year = value.parse().ok(),
                    Some("width") if in_video => parsed.resolution_width = value.parse().ok(),
                    Some("height") if in_video => parsed.resolution_height = value.parse().ok(),
                    Some("codec") if in_video => parsed.video_codec = Some(value.to_string()),
                    Some("hdr") if in_video => {
                        parsed.hdr = matches!(value.to_lowercase().as_str(), "true" | "yes" | "1");
                    }
                    Some("uniqueid") => {
                        let lowered = current_uniqueid_type.as_deref().unwrap_or("").to_lowercase();
                        if lowered == "imdb" {
                            parsed.imdb_id = Some(value.to_string());
                        } else if lowered == "tmdb" {
                            parsed.tmdb_id = Some(value.to_string());
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(tag)) => {
                let name = String::from_utf8_lossy(tag.name().as_ref()).to_string();
                if name == "video" {
                    in_video = false;
                }
                current_element = None;
                current_uniqueid_type = None;
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("NFO XML parse error: {e}")),
            _ => {}
        }
        buf.clear();
    }

    if parsed.title.is_none() {
        return Err("NFO missing title".to_string());
    }

    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_tmm_style_nfo() {
        let xml = r#"
        <movie>
          <title>Inception</title>
          <year>2010</year>
          <uniqueid type="imdb" default="true">tt1375666</uniqueid>
          <uniqueid type="tmdb">27205</uniqueid>
          <fileinfo>
            <streamdetails>
              <video>
                <width>1920</width>
                <height>1080</height>
                <codec>h264</codec>
              </video>
            </streamdetails>
          </fileinfo>
        </movie>
        "#;

        let parsed = parse_nfo_content(xml).unwrap();
        assert_eq!(parsed.title.as_deref(), Some("Inception"));
        assert_eq!(parsed.year, Some(2010));
        assert_eq!(parsed.imdb_id.as_deref(), Some("tt1375666"));
        assert_eq!(parsed.tmdb_id.as_deref(), Some("27205"));
        assert_eq!(parsed.resolution_height, Some(1080));
    }
}