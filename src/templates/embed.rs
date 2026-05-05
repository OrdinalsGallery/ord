use super::*;

#[derive(Boilerplate)]
pub(crate) struct EmbedAudioHtml {
  pub(crate) inscription_id: InscriptionId,
  pub(crate) content_type: String,
  pub(crate) is_opus: bool,
  pub(crate) title: String,
  pub(crate) metadata: Vec<(String, String)>,
}

#[derive(Boilerplate)]
pub(crate) struct EmbedImageHtml {
  pub(crate) inscription_id: InscriptionId,
  pub(crate) image_rendering: ImageRendering,
  pub(crate) title: String,
}

#[derive(Boilerplate)]
pub(crate) struct EmbedVideoHtml {
  pub(crate) inscription_id: InscriptionId,
  pub(crate) title: String,
}

#[derive(Boilerplate)]
pub(crate) struct EmbedUnknownHtml {
  pub(crate) inscription_id: InscriptionId,
  pub(crate) inscription_number: i32,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn embed_audio_html_renders_audio_element() {
    let id = inscription_id(1);
    let html = EmbedAudioHtml {
      inscription_id: id,
      content_type: "audio/flac".into(),
      is_opus: false,
      title: "My Song".into(),
      metadata: Vec::new(),
    }
    .to_string();

    assert!(html.contains("<title>My Song</title>"));
    assert!(html.contains("id=audio-player"));
    assert!(
      html.contains(&format!(r#"<source src=/content/{id} type="audio/flac">"#)),
      "missing audio source for {id}"
    );
    assert!(!html.contains("opus-stream-decoder"));
    assert!(!html.contains("embed-info-toggle"));
  }

  #[test]
  fn embed_audio_opus_includes_decoder() {
    let id = inscription_id(1);
    let html = EmbedAudioHtml {
      inscription_id: id,
      content_type: "audio/ogg;codecs=opus".into(),
      is_opus: true,
      title: "Inscription 1".into(),
      metadata: Vec::new(),
    }
    .to_string();

    assert!(html.contains("/static/opus-stream-decoder.js"));
    assert!(html.contains("/static/preview-audio.js"));
    assert!(html.contains("locateFile"));
    assert!(html.contains("Object.defineProperty(Module"));
    assert!(html.contains("id=audio-player"));
    assert!(html.contains(&format!(
      r#"<source src=/content/{id} type="audio/ogg;codecs=opus">"#
    )));
  }

  #[test]
  fn embed_audio_title_html_escapes() {
    let html = EmbedAudioHtml {
      inscription_id: inscription_id(1),
      content_type: "audio/flac".into(),
      is_opus: false,
      title: "<script>alert(1)</script>".into(),
      metadata: Vec::new(),
    }
    .to_string();

    assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
    assert!(!html.contains("<script>alert(1)</script>"));
  }

  #[test]
  fn embed_audio_renders_metadata_panel_when_entries_present() {
    let html = EmbedAudioHtml {
      inscription_id: inscription_id(1),
      content_type: "audio/ogg;codecs=opus".into(),
      is_opus: true,
      title: "Track".into(),
      metadata: vec![
        ("title".into(), "Track".into()),
        ("artist".into(), "Tatiana <Moroz>".into()),
        ("description".into(), "Long\ntext".into()),
      ],
    }
    .to_string();

    assert!(html.contains("class=embed-info-toggle"));
    assert!(html.contains("class=embed-metadata-panel"));
    assert!(html.contains("<dt>artist</dt>"));
    assert!(html.contains("Tatiana &lt;Moroz&gt;"));
    assert!(!html.contains("<Moroz>"));
  }

  #[test]
  fn embed_image_html_renders_img() {
    let id = inscription_id(1);
    let html = EmbedImageHtml {
      inscription_id: id,
      image_rendering: ImageRendering::Pixelated,
      title: "Inscription 1".into(),
    }
    .to_string();

    assert!(html.contains(&format!("<img src=/content/{id}")));
    assert!(html.contains("image-rendering: pixelated"));
  }

  #[test]
  fn embed_video_html_renders_video() {
    let id = inscription_id(1);
    let html = EmbedVideoHtml {
      inscription_id: id,
      title: "Inscription 1".into(),
    }
    .to_string();

    assert!(html.contains("<video"));
    assert!(html.contains(&format!("<source src=/content/{id}")));
  }

  #[test]
  fn embed_unknown_renders_static_message() {
    let id = inscription_id(1);
    let html = EmbedUnknownHtml {
      inscription_id: id,
      inscription_number: 1,
    }
    .to_string();

    assert!(html.contains(&format!("/inscription/{id}")));
    assert!(html.contains("not embeddable"));
    assert!(html.contains("Inscription 1"));
  }
}
