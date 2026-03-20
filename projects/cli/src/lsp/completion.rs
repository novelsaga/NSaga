#![allow(dead_code)]

use std::sync::LazyLock;

use jieba_rs::{Jieba, TokenizeMode};
use novelsaga_core::metadata::MetadataEntity;
use tower_lsp::lsp_types::{CompletionItem, CompletionItemKind};

static JIEBA: LazyLock<Jieba> = LazyLock::new(Jieba::new);
const MAX_COMPLETION_ITEMS: usize = 20;

pub fn extract_active_prefix(text: &str, cursor_offset: usize) -> String {
  if cursor_offset == 0 || cursor_offset > text.len() || !text.is_char_boundary(cursor_offset) {
    return String::new();
  }

  let line_start = text[..cursor_offset].rfind(['\n', '\r']).map_or(0, |index| index + 1);
  let line_prefix = &text[line_start..cursor_offset];
  let chunk_start = find_chunk_start(line_prefix);
  let active_chunk = &line_prefix[chunk_start..];

  if active_chunk.is_empty() {
    return String::new();
  }

  if !contains_cjk(active_chunk) {
    return active_chunk.to_string();
  }

  extract_cjk_suffix(active_chunk)
}

pub fn build_completion_candidates(entities: &[MetadataEntity], prefix: &str) -> Vec<CompletionItem> {
  let normalized_prefix = normalize_for_match(prefix);
  let mut candidates = entities
    .iter()
    .filter_map(|entity| Candidate::from_entity(entity, &normalized_prefix))
    .collect::<Vec<_>>();

  candidates.sort_by(|left, right| left.sort_key.cmp(&right.sort_key));
  candidates
    .into_iter()
    .take(MAX_COMPLETION_ITEMS)
    .map(|candidate| candidate.item)
    .collect()
}

fn find_chunk_start(text: &str) -> usize {
  text
    .char_indices()
    .rev()
    .find_map(|(index, ch)| is_completion_boundary(ch).then_some(index + ch.len_utf8()))
    .unwrap_or(0)
}

fn is_completion_boundary(ch: char) -> bool {
  ch.is_whitespace() || (!ch.is_alphanumeric() && ch != '_' && ch != '-')
}

fn contains_cjk(text: &str) -> bool {
  text.chars().any(is_cjk)
}

fn is_cjk(ch: char) -> bool {
  matches!(
    ch as u32,
    0x3400..=0x4DBF
      | 0x4E00..=0x9FFF
      | 0xF900..=0xFAFF
      | 0x20000..=0x2A6DF
      | 0x2A700..=0x2B73F
      | 0x2B740..=0x2B81F
      | 0x2B820..=0x2CEAF
      | 0x2CEB0..=0x2EBEF
      | 0x2F800..=0x2FA1F
  )
}

fn extract_cjk_suffix(text: &str) -> String {
  let tokens = JIEBA.tokenize(text, TokenizeMode::Default, true);
  let best_start = tokens
    .get(tokens.len().saturating_sub(2))
    .map_or(0, |token| token.start);

  text[char_to_byte_index(text, best_start)..].to_string()
}

fn char_to_byte_index(text: &str, char_offset: usize) -> usize {
  if char_offset == 0 {
    return 0;
  }

  text
    .char_indices()
    .nth(char_offset)
    .map_or(text.len(), |(index, _)| index)
}

fn normalize_for_match(text: &str) -> String {
  text
    .chars()
    .filter(|ch| !ch.is_whitespace() && *ch != '_' && *ch != '-')
    .flat_map(char::to_lowercase)
    .collect()
}

fn entity_label(entity: &MetadataEntity) -> String {
  ["name", "title"]
    .into_iter()
    .find_map(|field| entity.get_field(field).and_then(|value| value.as_str()))
    .map_or_else(|| entity.id.clone(), ToString::to_string)
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CandidateSortKey {
  match_priority: u8,
  match_index: usize,
  normalized_label: String,
  namespace: String,
  type_: String,
  id: String,
}

struct Candidate {
  item: CompletionItem,
  sort_key: CandidateSortKey,
}

impl Candidate {
  fn from_entity(entity: &MetadataEntity, normalized_prefix: &str) -> Option<Self> {
    let label = entity_label(entity);
    let normalized_label = normalize_for_match(&label);
    let normalized_id = normalize_for_match(&entity.id);
    let normalized_title = entity
      .get_field("title")
      .and_then(|value| value.as_str())
      .map(normalize_for_match);
    let normalized_name = entity
      .get_field("name")
      .and_then(|value| value.as_str())
      .map(normalize_for_match);

    let match_quality = MatchQuality::for_fields(
      normalized_prefix,
      [&normalized_label, &normalized_id]
        .into_iter()
        .chain(normalized_name.iter())
        .chain(normalized_title.iter()),
    )?;

    Some(Self {
      item: CompletionItem {
        label: label.clone(),
        insert_text: Some(label.clone()),
        detail: Some(format!("{} · {}", entity.type_, entity.namespace)),
        kind: Some(CompletionItemKind::REFERENCE),
        ..CompletionItem::default()
      },
      sort_key: CandidateSortKey {
        match_priority: match_quality.priority,
        match_index: match_quality.index,
        normalized_label,
        namespace: entity.namespace.clone(),
        type_: entity.type_.clone(),
        id: entity.id.clone(),
      },
    })
  }
}

#[derive(Debug, Clone, Copy)]
struct MatchQuality {
  priority: u8,
  index: usize,
}

impl MatchQuality {
  fn for_fields<'a>(prefix: &str, fields: impl IntoIterator<Item = &'a String>) -> Option<Self> {
    if prefix.is_empty() {
      return Some(Self { priority: 0, index: 0 });
    }

    fields
      .into_iter()
      .filter_map(|field| Self::for_field(prefix, field))
      .min_by_key(|quality| (quality.priority, quality.index))
  }

  fn for_field(prefix: &str, field: &str) -> Option<Self> {
    if field.starts_with(prefix) {
      Some(Self { priority: 0, index: 0 })
    } else {
      field.find(prefix).map(|index| Self { priority: 1, index })
    }
  }
}

#[cfg(test)]
mod tests {
  use serde_json::json;

  use super::*;

  fn entity(id: &str, type_: &str, namespace: &str, title: Option<&str>) -> MetadataEntity {
    let frontmatter = title.map_or_else(|| json!({}), |title| json!({ "title": title }));
    MetadataEntity::new(id, type_, namespace, frontmatter, "")
  }

  #[test]
  fn extract_active_prefix_from_contiguous_cjk_text() {
    let text = "前缀英雄王者\n第二行";
    let cursor_offset = "前缀英雄王者".len();

    assert_eq!(extract_active_prefix(text, cursor_offset), "英雄王者");
  }

  #[test]
  fn extract_active_prefix_from_ascii_text() {
    let text = "hero_alpha-beta detail";
    let cursor_offset = "hero_alpha-beta".len();

    assert_eq!(extract_active_prefix(text, cursor_offset), "hero_alpha-beta");
  }

  #[test]
  fn rank_candidates_is_deterministic() {
    let entities = vec![
      entity("hero-beta", "character", "cast", Some("Hero Beta")),
      entity("hero-alpha", "character", "cast", Some("Hero Alpha")),
      entity("sidekick", "character", "cast", Some("Alpha Sidekick")),
      entity("villain", "character", "cast", Some("Zeta Villain")),
    ];

    let items = build_completion_candidates(&entities, "hero");
    let labels: Vec<_> = items.iter().map(|item| item.label.as_str()).collect();
    let insert_texts: Vec<_> = items.iter().map(|item| item.insert_text.as_deref()).collect();
    let details: Vec<_> = items.iter().map(|item| item.detail.as_deref()).collect();

    assert_eq!(labels, vec!["Hero Alpha", "Hero Beta"]);
    assert_eq!(insert_texts, vec![Some("Hero Alpha"), Some("Hero Beta")]);
    assert_eq!(details, vec![Some("character · cast"), Some("character · cast")]);
    assert!(
      items
        .iter()
        .all(|item| item.kind == Some(CompletionItemKind::REFERENCE))
    );
  }

  #[test]
  fn rank_candidates_caps_results_at_twenty() {
    let entities = (0..25)
      .map(|index| {
        entity(
          &format!("hero-{index:02}"),
          "character",
          "cast",
          Some(&format!("Hero {index:02}")),
        )
      })
      .collect::<Vec<_>>();

    let items = build_completion_candidates(&entities, "hero");
    let labels: Vec<_> = items.iter().map(|item| item.label.as_str()).collect();

    assert_eq!(items.len(), 20);
    assert_eq!(labels.first().copied(), Some("Hero 00"));
    assert_eq!(labels.last().copied(), Some("Hero 19"));
  }
}
