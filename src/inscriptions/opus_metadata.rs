use super::*;

const SANITIZED_TAG_MAX_LEN: usize = 200;

const COMMON_FIELDS: [&str; 18] = [
  "title",
  "artist",
  "album",
  "album artist",
  "date",
  "track number",
  "genre",
  "composer",
  "copyright",
  "description",
  "producer",
  "engineer",
  "mixer",
  "assistant engineer",
  "songwriter",
  "mastering",
  "inscribing",
  "technology",
];

pub(crate) fn is_opus_content_type(content_type: &str) -> bool {
  let lower = content_type.to_ascii_lowercase();
  lower == "audio/opus"
    || lower.starts_with("audio/opus;")
    || (lower.contains("audio/ogg") && lower.contains("opus"))
}

pub(crate) fn raw_tags(body: &[u8]) -> Vec<(String, String)> {
  use symphonia::core::{
    formats::FormatOptions, io::MediaSourceStream, meta::MetadataOptions, probe::Hint,
  };

  let media_source = MediaSourceStream::new(
    Box::new(Cursor::new(body.to_vec())),
    Default::default(),
  );

  let mut hint = Hint::new();
  hint.with_extension("ogg");

  let Ok(mut probed) = symphonia::default::get_probe().format(
    &hint,
    media_source,
    &FormatOptions::default(),
    &MetadataOptions::default(),
  ) else {
    return Vec::new();
  };

  let metadata_queue = probed.format.metadata();
  let Some(metadata) = metadata_queue.current() else {
    return Vec::new();
  };

  metadata
    .tags()
    .iter()
    .filter_map(|tag| {
      let key = tag.key.trim();
      if key.is_empty() || is_picture_tag(key) {
        return None;
      }
      Some((key.to_string(), tag.value.to_string()))
    })
    .collect()
}

pub(crate) fn title(body: &[u8]) -> Option<String> {
  find_tag(body, "title")
}

pub(crate) fn artist(body: &[u8]) -> Option<String> {
  find_tag(body, "artist")
}

fn find_tag(body: &[u8], target: &str) -> Option<String> {
  raw_tags(body).into_iter().find_map(|(key, value)| {
    if key.eq_ignore_ascii_case(target) {
      let sanitized = sanitize_tag(value);
      if sanitized.is_empty() {
        None
      } else {
        Some(sanitized)
      }
    } else {
      None
    }
  })
}

pub(crate) fn sanitize_tag(raw: String) -> String {
  let trimmed: String = raw
    .chars()
    .filter(|c| !c.is_ascii_control())
    .collect::<String>()
    .trim()
    .to_string();

  if trimmed.chars().count() <= SANITIZED_TAG_MAX_LEN {
    return trimmed;
  }

  trimmed.chars().take(SANITIZED_TAG_MAX_LEN).collect()
}

pub(crate) fn is_picture_tag(key: &str) -> bool {
  let upper = key.to_ascii_uppercase();
  upper == "METADATA_BLOCK_PICTURE" || upper.contains("PICTURE")
}

pub(crate) fn structured(body: &[u8]) -> Option<Value> {
  from_tags(raw_tags(body))
}

pub(crate) fn from_tags(tags: Vec<(String, String)>) -> Option<Value> {
  let mut grouped = BTreeMap::<String, Vec<String>>::new();

  for (key, value) in tags {
    if is_picture_tag(&key) {
      continue;
    }

    let normalized_key = normalize_opus_tag_key(&key);
    if normalized_key.is_empty() {
      continue;
    }

    grouped.entry(normalized_key).or_default().push(value);
  }

  if grouped.is_empty() {
    return None;
  }

  let mut map = Vec::new();

  for key in COMMON_FIELDS {
    if let Some(values) = grouped.remove(key) {
      map.push((Value::Text(key.into()), values_to_value(values)));
    }
  }

  for (key, values) in grouped {
    map.push((Value::Text(key), values_to_value(values)));
  }

  Some(Value::Map(map))
}

fn values_to_value(values: Vec<String>) -> Value {
  if values.len() == 1 {
    return Value::Text(values.into_iter().next().unwrap_or_default());
  }
  Value::Array(values.into_iter().map(Value::Text).collect())
}

pub(crate) fn normalize_opus_tag_key(key: &str) -> String {
  let mut expanded = String::new();
  let mut previous_was_lowercase = false;

  for c in key.trim().chars() {
    if c.is_ascii_uppercase() && previous_was_lowercase {
      expanded.push(' ');
    }

    if matches!(c, '_' | '-' | '.') {
      expanded.push(' ');
    } else {
      expanded.push(c);
    }

    previous_was_lowercase = c.is_ascii_lowercase();
  }

  let normalized = expanded
    .split_whitespace()
    .map(|part| part.to_ascii_lowercase())
    .collect::<Vec<String>>()
    .join(" ");

  let compact = normalized.replace(' ', "");

  match compact.as_str() {
    "title" => "title".into(),
    "artist" => "artist".into(),
    "album" => "album".into(),
    "albumartist" => "album artist".into(),
    "date" | "year" => "date".into(),
    "track" | "tracknumber" => "track number".into(),
    "genre" => "genre".into(),
    "composer" => "composer".into(),
    "copyright" => "copyright".into(),
    "description" | "comment" => "description".into(),
    "producer" => "producer".into(),
    "engineer" => "engineer".into(),
    "mixer" => "mixer".into(),
    "assistantengineer" => "assistant engineer".into(),
    "songwriter" | "lyricist" => "songwriter".into(),
    "mastering" => "mastering".into(),
    "inscribing" => "inscribing".into(),
    "technology" => "technology".into(),
    _ => normalized,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const COMINGSOON_OPUS: &[u8] = include_bytes!("../../testdata/comingsoon.opus");

  #[test]
  fn title_extracts_simple_title_tag() {
    assert_eq!(title(COMINGSOON_OPUS).as_deref(), Some("comingsoon"));
  }

  #[test]
  fn title_returns_none_for_non_opus() {
    assert!(title(b"random non-opus bytes").is_none());
    assert!(title(&[0u8; 32]).is_none());
  }

  #[test]
  fn title_handles_case_insensitive_keys() {
    let value = find_tag_in(
      vec![("TiTlE".into(), "x".into())],
      "title",
    );
    assert_eq!(value.as_deref(), Some("x"));
  }

  #[test]
  fn title_strips_whitespace_only() {
    let value = find_tag_in(
      vec![("title".into(), "   ".into())],
      "title",
    );
    assert!(value.is_none());
  }

  #[test]
  fn raw_tags_excludes_pictures() {
    assert!(
      !raw_tags(COMINGSOON_OPUS)
        .iter()
        .any(|(key, _)| is_picture_tag(key))
    );
  }

  #[test]
  fn sanitize_tag_strips_control_chars_and_truncates() {
    assert_eq!(sanitize_tag("hi\x00\x07\x1bthere".into()), "hithere");
    assert_eq!(sanitize_tag("  hello  ".into()), "hello");

    let long: String = "a".repeat(SANITIZED_TAG_MAX_LEN + 50);
    assert_eq!(
      sanitize_tag(long).chars().count(),
      SANITIZED_TAG_MAX_LEN
    );
  }

  #[test]
  fn is_opus_content_type_matches_variants() {
    assert!(is_opus_content_type("audio/opus"));
    assert!(is_opus_content_type("audio/opus; charset=utf-8"));
    assert!(is_opus_content_type("audio/ogg;codecs=opus"));
    assert!(is_opus_content_type("AUDIO/OGG;CODECS=OPUS"));
    assert!(!is_opus_content_type("audio/ogg"));
    assert!(!is_opus_content_type("audio/flac"));
  }

  fn find_tag_in(tags: Vec<(String, String)>, target: &str) -> Option<String> {
    tags.into_iter().find_map(|(key, value)| {
      if key.eq_ignore_ascii_case(target) {
        let sanitized = sanitize_tag(value);
        if sanitized.is_empty() {
          None
        } else {
          Some(sanitized)
        }
      } else {
        None
      }
    })
  }
}
