use super::*;

#[derive(Clone, Copy, Debug, Deserialize, Default, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Sort {
  #[default]
  Newest,
  Oldest,
}

impl std::fmt::Display for Sort {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    f.write_str(match self {
      Sort::Newest => "newest",
      Sort::Oldest => "oldest",
    })
  }
}

#[derive(Boilerplate)]
pub(crate) struct InscriptionsHtml {
  pub(crate) inscriptions: Vec<InscriptionId>,
  pub(crate) prev: Option<u32>,
  pub(crate) next: Option<u32>,
  pub(crate) sort: Sort,
}

impl InscriptionsHtml {
  pub(crate) fn sort_query(&self) -> &'static str {
    match self.sort {
      Sort::Newest => "",
      Sort::Oldest => "?sort=oldest",
    }
  }

  pub(crate) fn selected_if(&self, sort: Sort) -> &'static str {
    if self.sort == sort { " selected" } else { "" }
  }
}

impl PageContent for InscriptionsHtml {
  fn title(&self) -> String {
    "Inscriptions".into()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn without_prev_and_next() {
    assert_regex_match!(
      InscriptionsHtml {
        inscriptions: vec![inscription_id(1), inscription_id(2)],
        prev: None,
        next: None,
        sort: Sort::Newest,
      },
      "
        <h1>All Inscriptions</h1>
        <form class=sort-form action=/inscriptions method=get>.*</form>
        <div class=thumbnails>
          <a href=/inscription/1{64}i1><iframe .* src=/preview/1{64}i1\\?thumb=1></iframe></a>
          <a href=/inscription/2{64}i2><iframe .* src=/preview/2{64}i2\\?thumb=1></iframe></a>
        </div>
        .*
        prev
        next
        .*
      "
      .unindent()
    );
  }

  #[test]
  fn with_prev_and_next() {
    assert_regex_match!(
      InscriptionsHtml {
        inscriptions: vec![inscription_id(1), inscription_id(2)],
        prev: Some(1),
        next: Some(2),
        sort: Sort::Newest,
      },
      "
        <h1>All Inscriptions</h1>
        <form class=sort-form action=/inscriptions method=get>.*</form>
        <div class=thumbnails>
          <a href=/inscription/1{64}i1><iframe .* src=/preview/1{64}i1\\?thumb=1></iframe></a>
          <a href=/inscription/2{64}i2><iframe .* src=/preview/2{64}i2\\?thumb=1></iframe></a>
        </div>
        .*
        <a class=prev href=/inscriptions/1>prev</a>
        <a class=next href=/inscriptions/2>next</a>
        .*
      "
      .unindent()
    );
  }

  #[test]
  fn oldest_sort_preserved_in_pagination_links() {
    assert_regex_match!(
      InscriptionsHtml {
        inscriptions: vec![inscription_id(1)],
        prev: Some(0),
        next: Some(2),
        sort: Sort::Oldest,
      },
      "
        <h1>All Inscriptions</h1>
        .*
        <a class=prev href=/inscriptions/0\\?sort=oldest>prev</a>
        <a class=next href=/inscriptions/2\\?sort=oldest>next</a>
        .*
      "
      .unindent()
    );
  }
}
