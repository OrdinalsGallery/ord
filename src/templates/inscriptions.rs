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

impl Sort {
  pub fn label(self) -> &'static str {
    match self {
      Sort::Newest => "Recently Inscribed",
      Sort::Oldest => "Earliest Inscribed",
    }
  }
}

#[derive(Clone, Copy, Debug, Deserialize, Default, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Rarity {
  #[default]
  Any,
  Uncommon,
  Rare,
  Epic,
  Legendary,
  Mythic,
}

impl std::fmt::Display for Rarity {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    f.write_str(match self {
      Rarity::Any => "any",
      Rarity::Uncommon => "uncommon",
      Rarity::Rare => "rare",
      Rarity::Epic => "epic",
      Rarity::Legendary => "legendary",
      Rarity::Mythic => "mythic",
    })
  }
}

impl Rarity {
  fn charm(self) -> Option<ordinals::Charm> {
    match self {
      Rarity::Any => None,
      Rarity::Uncommon => Some(ordinals::Charm::Uncommon),
      Rarity::Rare => Some(ordinals::Charm::Rare),
      Rarity::Epic => Some(ordinals::Charm::Epic),
      Rarity::Legendary => Some(ordinals::Charm::Legendary),
      Rarity::Mythic => Some(ordinals::Charm::Mythic),
    }
  }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Filter {
  pub cursed: bool,
  pub from: Option<i32>,
  pub to: Option<i32>,
  pub rarity: Rarity,
}

impl Filter {
  pub fn is_unfiltered(&self) -> bool {
    !self.cursed && self.from.is_none() && self.to.is_none() && self.rarity == Rarity::Any
  }

  pub fn matches(&self, entry: &crate::index::entry::InscriptionEntry) -> bool {
    if self.cursed && entry.inscription_number >= 0 {
      return false;
    }
    if let Some(from) = self.from {
      if entry.inscription_number < from {
        return false;
      }
    }
    if let Some(to) = self.to {
      if entry.inscription_number > to {
        return false;
      }
    }
    if let Some(charm) = self.rarity.charm() {
      if !charm.is_set(entry.charms) {
        return false;
      }
    }
    true
  }
}

#[derive(Boilerplate)]
pub(crate) struct InscriptionsHtml {
  pub(crate) inscriptions: Vec<InscriptionId>,
  pub(crate) prev: Option<u32>,
  pub(crate) next: Option<u32>,
  pub(crate) sort: Sort,
  pub(crate) filter: Filter,
  pub(crate) index_sats: bool,
}

impl InscriptionsHtml {
  // Returns the query string (without leading "?") preserving all active
  // filters and the sort. Used for pagination prev/next links.
  pub(crate) fn pagination_query(&self) -> String {
    let mut parts: Vec<String> = Vec::new();
    if self.sort != Sort::Newest {
      parts.push(format!("sort={}", self.sort));
    }
    if self.filter.cursed {
      parts.push("cursed=1".into());
    }
    if let Some(from) = self.filter.from {
      parts.push(format!("from={from}"));
    }
    if let Some(to) = self.filter.to {
      parts.push(format!("to={to}"));
    }
    if self.filter.rarity != Rarity::Any {
      parts.push(format!("rarity={}", self.filter.rarity));
    }
    if parts.is_empty() {
      String::new()
    } else {
      format!("?{}", parts.join("&"))
    }
  }

  pub(crate) fn selected_sort_if(&self, sort: Sort) -> &'static str {
    if self.sort == sort { " selected" } else { "" }
  }

  pub(crate) fn selected_rarity_if(&self, rarity: Rarity) -> &'static str {
    if self.filter.rarity == rarity { " selected" } else { "" }
  }

  pub(crate) fn cursed_class(&self) -> &'static str {
    if self.filter.cursed { "active" } else { "" }
  }

  pub(crate) fn cursed_checked(&self) -> &'static str {
    if self.filter.cursed { " checked" } else { "" }
  }

  pub(crate) fn from_value(&self) -> String {
    self.filter.from.map(|n| n.to_string()).unwrap_or_default()
  }

  pub(crate) fn to_value(&self) -> String {
    self.filter.to.map(|n| n.to_string()).unwrap_or_default()
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

  fn html(sort: Sort, filter: Filter, prev: Option<u32>, next: Option<u32>) -> InscriptionsHtml {
    InscriptionsHtml {
      inscriptions: vec![inscription_id(1), inscription_id(2)],
      prev,
      next,
      sort,
      filter,
      index_sats: false,
    }
  }

  #[test]
  fn without_prev_and_next() {
    assert_regex_match!(
      html(Sort::Newest, Filter::default(), None, None),
      "
        .*<h1>All Inscriptions</h1>.*
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
      html(Sort::Newest, Filter::default(), Some(1), Some(2)),
      "
        .*<a class=prev href=/inscriptions/1>prev</a>
        <a class=next href=/inscriptions/2>next</a>.*
      "
      .unindent()
    );
  }

  #[test]
  fn oldest_sort_preserved_in_pagination_links() {
    assert_regex_match!(
      html(Sort::Oldest, Filter::default(), Some(0), Some(2)),
      "
        .*<a class=prev href=/inscriptions/0\\?sort=oldest>prev</a>
        <a class=next href=/inscriptions/2\\?sort=oldest>next</a>.*
      "
      .unindent()
    );
  }

  #[test]
  fn filters_preserved_in_pagination_links() {
    let filter = Filter {
      cursed: true,
      from: Some(100),
      to: Some(200),
      rarity: Rarity::Uncommon,
    };
    assert_regex_match!(
      html(Sort::Newest, filter, Some(1), Some(2)),
      "
        .*<a class=prev href=/inscriptions/1\\?cursed=1&from=100&to=200&rarity=uncommon>prev</a>.*
      "
      .unindent()
    );
  }
}
