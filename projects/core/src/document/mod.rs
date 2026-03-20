//! Document module - shared parsing layer for markdown files
//!
//! This module provides types and utilities for parsing and classifying markdown documents
//! used by both articles and metadata entities.

use std::path::Path;

use serde_json::{Value, json};

use crate::{article::ArticleDocument, metadata::MetadataEntity};

/// Parsed markdown document with separated frontmatter and body
///
/// This is the shared parsing layer used by both articles and metadata documents.
/// It separates YAML frontmatter from the markdown body content.
#[derive(Debug, Clone)]
pub struct MarkdownParts {
  /// Parsed frontmatter as JSON value
  pub frontmatter: Value,
  /// Markdown body content (everything after frontmatter)
  pub body: String,
  /// Whether the document had frontmatter delimiters
  pub has_frontmatter: bool,
}

/// Severity level for markdown parse issues.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseSeverity {
  /// Parsing cannot fully trust the source content.
  Error,
  /// Parsing succeeded but found a suspicious value.
  Warning,
}

/// Diagnostic issue found while parsing markdown frontmatter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseIssue {
  /// Severity of the issue.
  pub severity: ParseSeverity,
  /// Human-readable issue description.
  pub message: String,
  /// 1-based line number in the source document when known.
  pub line: Option<usize>,
}

/// Parsed markdown plus any frontmatter diagnostics.
#[derive(Debug, Clone)]
pub struct MarkdownParseReport {
  /// Parsed markdown parts.
  pub parts: MarkdownParts,
  /// Collected parse issues.
  pub issues: Vec<ParseIssue>,
}

impl MarkdownParts {
  /// Create a new `MarkdownParts` with parsed frontmatter and body
  ///
  /// # Arguments
  /// * `frontmatter` - Parsed YAML frontmatter as JSON value
  /// * `body` - Markdown body content
  /// * `has_frontmatter` - Whether document had frontmatter delimiters
  ///
  /// # Returns
  /// New `MarkdownParts` instance
  #[must_use]
  pub fn new(frontmatter: Value, body: String, has_frontmatter: bool) -> Self {
    Self {
      frontmatter,
      body,
      has_frontmatter,
    }
  }

  /// Parse markdown content into `MarkdownParts`
  ///
  /// Splits document at `---` delimiters (YAML frontmatter format).
  /// If no frontmatter delimiters found, entire content is treated as body.
  ///
  /// # Arguments
  /// * `content` - Raw markdown file content
  ///
  /// # Returns
  /// Parsed `MarkdownParts` with separated frontmatter and body
  ///
  /// # Examples
  /// ```ignore
  /// let content = "---\ntitle: Hello\n---\n# Body";
  /// let parts = MarkdownParts::parse(content);
  /// assert!(parts.has_frontmatter);
  /// assert_eq!(parts.body, "# Body");
  /// ```
  #[must_use]
  pub fn parse(content: &str) -> Self {
    Self::parse_with_issues(content).parts
  }

  /// Parse markdown content into `MarkdownParts` and collect frontmatter issues.
  #[must_use]
  pub fn parse_with_issues(content: &str) -> MarkdownParseReport {
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() || !lines[0].trim().starts_with("---") {
      return MarkdownParseReport {
        parts: markdown_without_frontmatter(content),
        issues: Vec::new(),
      };
    }

    let closing_delimiter_idx = lines[1..]
      .iter()
      .position(|line| line.trim().starts_with("---"))
      .map(|idx| idx + 1);

    let Some(delimiter_idx) = closing_delimiter_idx else {
      return MarkdownParseReport {
        parts: markdown_without_frontmatter(content),
        issues: vec![ParseIssue {
          severity: ParseSeverity::Error,
          message: "Unterminated frontmatter: missing closing delimiter".to_string(),
          line: Some(1),
        }],
      };
    };

    let frontmatter_lines = &lines[1..delimiter_idx];
    let frontmatter = parse_yaml_to_json(&frontmatter_lines.join("\n"));
    let body_start = delimiter_idx + 1;
    let body = if body_start < lines.len() {
      lines[body_start..].join("\n")
    } else {
      String::new()
    };

    MarkdownParseReport {
      parts: Self {
        frontmatter: frontmatter.clone(),
        body,
        has_frontmatter: true,
      },
      issues: collect_frontmatter_issues(frontmatter_lines, &frontmatter),
    }
  }

  /// Get a field from frontmatter
  ///
  /// # Arguments
  /// * `key` - Field name to retrieve
  ///
  /// # Returns
  /// Reference to the value if present, None otherwise
  #[must_use]
  pub fn get_field(&self, key: &str) -> Option<&Value> {
    self.frontmatter.get(key)
  }
}

/// Classification of document type based on file path
///
/// Used to determine how to interpret and process markdown documents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentKind {
  /// Metadata document - in metadata/** directories
  Metadata,
  /// Article document - regular content files
  Article,
}

impl DocumentKind {
  /// Classify a document by its file path
  ///
  /// # Rules
  /// - Paths matching `**/metadata/**/*.md` → Metadata
  /// - All other `.md` files → Article
  ///
  /// # Arguments
  /// * `path` - File path to classify
  ///
  /// # Returns
  /// Classified `DocumentKind`
  ///
  /// # Examples
  /// ```ignore
  /// let metadata_doc = Path::new("metadata/characters/hero.md");
  /// assert_eq!(DocumentKind::classify_path(metadata_doc), DocumentKind::Metadata);
  ///
  /// let article_doc = Path::new("chapters/chapter-01.md");
  /// assert_eq!(DocumentKind::classify_path(article_doc), DocumentKind::Article);
  /// ```
  #[must_use]
  pub fn classify_path(path: &Path) -> Self {
    // Check if path contains "metadata" as a directory component
    for component in path.components() {
      if let std::path::Component::Normal(os_str) = component
        && os_str.to_string_lossy() == "metadata"
      {
        return DocumentKind::Metadata;
      }
    }
    DocumentKind::Article
  }
}

/// Unified document type for both metadata and article documents
///
/// `WorkspaceDocument` is the entry point for the LSP layer, providing a single type
/// that can represent either a metadata entity or an article document. This enables
/// uniform handling of different document types through pattern matching.
#[derive(Debug, Clone)]
pub enum WorkspaceDocument {
  /// Metadata entity document
  Metadata(MetadataEntity),
  /// Article document
  Article(ArticleDocument),
}

impl WorkspaceDocument {
  /// Get the document kind (Metadata or Article)
  ///
  /// # Returns
  /// The `DocumentKind` variant matching this document's type
  #[must_use]
  pub fn kind(&self) -> DocumentKind {
    match self {
      Self::Metadata(_) => DocumentKind::Metadata,
      Self::Article(_) => DocumentKind::Article,
    }
  }

  /// Try to get a reference to the metadata entity
  ///
  /// # Returns
  /// * `Some(&MetadataEntity)` if this is a Metadata variant
  /// * `None` if this is an Article variant
  #[must_use]
  pub fn as_metadata(&self) -> Option<&MetadataEntity> {
    match self {
      Self::Metadata(entity) => Some(entity),
      Self::Article(_) => None,
    }
  }

  /// Try to get a reference to the article document
  ///
  /// # Returns
  /// * `Some(&ArticleDocument)` if this is an Article variant
  /// * `None` if this is a Metadata variant
  #[must_use]
  pub fn as_article(&self) -> Option<&ArticleDocument> {
    match self {
      Self::Metadata(_) => None,
      Self::Article(article) => Some(article),
    }
  }

  /// Check if this is a metadata document
  ///
  /// # Returns
  /// true if this is a Metadata variant, false otherwise
  #[must_use]
  pub fn is_metadata(&self) -> bool {
    matches!(self, Self::Metadata(_))
  }

  /// Check if this is an article document
  ///
  /// # Returns
  /// true if this is an Article variant, false otherwise
  #[must_use]
  pub fn is_article(&self) -> bool {
    matches!(self, Self::Article(_))
  }
}

fn markdown_without_frontmatter(content: &str) -> MarkdownParts {
  MarkdownParts {
    frontmatter: json!({}),
    body: content.to_string(),
    has_frontmatter: false,
  }
}

fn collect_frontmatter_issues(frontmatter_lines: &[&str], frontmatter: &Value) -> Vec<ParseIssue> {
  let mut issues = Vec::new();
  let mut type_line = None;

  for (idx, line) in frontmatter_lines.iter().enumerate() {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
      continue;
    }

    if let Some((key, _)) = trimmed.split_once(':') {
      if key.trim().is_empty() {
        issues.push(ParseIssue {
          severity: ParseSeverity::Error,
          message: format!("Malformed frontmatter line: {trimmed}"),
          line: Some(idx + 2),
        });
        continue;
      }

      if key.trim() == "type" {
        type_line = Some(idx + 2);
      }
      continue;
    }

    issues.push(ParseIssue {
      severity: ParseSeverity::Error,
      message: format!("Malformed frontmatter line: {trimmed}"),
      line: Some(idx + 2),
    });
  }

  if frontmatter.get("type").is_some_and(|value| !value.is_string()) {
    issues.push(ParseIssue {
      severity: ParseSeverity::Warning,
      message: "Frontmatter 'type' field should be a string".to_string(),
      line: type_line,
    });
  }

  issues
}

/// Parse YAML string to JSON Value
///
/// Uses the config crate's YAML parser (via the config crate) to parse YAML
/// into a hashmap, then converts to `serde_json::Value` for consistency with
/// the rest of the codebase.
///
/// # Arguments
/// * `yaml_str` - YAML string to parse
///
/// # Returns
/// Parsed content as JSON Value, or empty object if parsing fails
fn parse_yaml_to_json(yaml_str: &str) -> Value {
  // Try to parse using serde_yaml -> serde_json conversion
  // First attempt: try parsing as YAML with serde_yaml if available
  use std::collections::BTreeMap;

  // Simple YAML parser using split + key:value parsing
  // For more complex YAML, this would need a full parser
  let mut result: BTreeMap<String, Value> = BTreeMap::new();

  for line in yaml_str.lines() {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
      continue;
    }

    if let Some((key, value)) = trimmed.split_once(':') {
      let key = key.trim().to_string();
      let value_str = value.trim();

      let parsed_value = match value_str {
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        "null" | "" => Value::Null,
        s if s.starts_with('"') && s.ends_with('"') => Value::String(s[1..s.len() - 1].to_string()),
        s if s.starts_with('\'') && s.ends_with('\'') => Value::String(s[1..s.len() - 1].to_string()),
        s if let Ok(num) = s.parse::<i64>() => Value::Number(num.into()),
        s if let Ok(num) = s.parse::<f64>() => {
          if let Some(n) = serde_json::Number::from_f64(num) {
            Value::Number(n)
          } else {
            Value::String(s.to_string())
          }
        }
        s => Value::String(s.to_string()),
      };

      result.insert(key, parsed_value);
    }
  }

  Value::Object(result.into_iter().collect::<serde_json::Map<String, Value>>())
}

#[cfg(test)]
mod tests {
  use super::*;

  // MarkdownParts tests
  #[test]
  fn test_parse_with_frontmatter() {
    let content = "---\ntitle: Hello\nauthor: Jane\n---\n# Body";
    let parts = MarkdownParts::parse(content);

    assert!(parts.has_frontmatter);
    assert_eq!(parts.body, "# Body");
    assert_eq!(parts.frontmatter.get("title").and_then(|v| v.as_str()), Some("Hello"));
    assert_eq!(parts.frontmatter.get("author").and_then(|v| v.as_str()), Some("Jane"));
  }

  #[test]
  fn test_parse_without_frontmatter() {
    let content = "# Just a header\n\nSome body";
    let parts = MarkdownParts::parse(content);

    assert!(!parts.has_frontmatter);
    assert_eq!(parts.body, content);
    assert!(parts.frontmatter.is_object());
  }

  #[test]
  fn test_parse_empty_frontmatter() {
    let content = "---\n---\n# Body";
    let parts = MarkdownParts::parse(content);

    assert!(parts.has_frontmatter);
    assert_eq!(parts.body, "# Body");
  }

  #[test]
  fn test_parse_only_opening_delimiter() {
    let content = "---\ntitle: Hello\n# No closing delimiter";
    let parts = MarkdownParts::parse(content);

    assert!(!parts.has_frontmatter);
    assert_eq!(parts.body, content);
  }

  #[test]
  fn parse_with_issues_valid_frontmatter_has_no_issues() {
    let content = "---\ntitle: Hello\ntype: article\n---\n# Body";
    let report = MarkdownParts::parse_with_issues(content);

    assert!(report.parts.has_frontmatter);
    assert_eq!(report.parts.body, "# Body");
    assert!(report.issues.is_empty());
  }

  #[test]
  fn parse_with_issues_reports_unterminated_frontmatter() {
    let content = "---\ntitle: Hello\nBody text here";
    let report = MarkdownParts::parse_with_issues(content);

    assert!(!report.parts.has_frontmatter);
    assert_eq!(report.parts.body, content);
    assert_eq!(report.issues.len(), 1);
    assert_eq!(report.issues[0].severity, ParseSeverity::Error);
    assert_eq!(
      report.issues[0].message,
      "Unterminated frontmatter: missing closing delimiter"
    );
  }

  #[test]
  fn parse_with_issues_reports_malformed_frontmatter_line() {
    let content = "---\ntitle: Hello\nthis is not a key value pair\n---\nBody";
    let report = MarkdownParts::parse_with_issues(content);

    assert!(report.parts.has_frontmatter);
    assert_eq!(report.parts.body, "Body");
    assert_eq!(
      report.parts.frontmatter.get("title").and_then(|v| v.as_str()),
      Some("Hello")
    );
    assert_eq!(report.issues.len(), 1);
    assert_eq!(report.issues[0].severity, ParseSeverity::Error);
    assert_eq!(
      report.issues[0].message,
      "Malformed frontmatter line: this is not a key value pair"
    );
  }

  #[test]
  fn parse_with_issues_reports_non_string_explicit_type() {
    let content = "---\ntitle: Hello\ntype: 123\n---\nBody";
    let report = MarkdownParts::parse_with_issues(content);

    assert!(report.parts.has_frontmatter);
    assert_eq!(
      report.parts.frontmatter.get("type").and_then(serde_json::Value::as_i64),
      Some(123)
    );
    assert_eq!(report.issues.len(), 1);
    assert_eq!(report.issues[0].severity, ParseSeverity::Warning);
    assert_eq!(report.issues[0].message, "Frontmatter 'type' field should be a string");
  }

  #[test]
  fn test_parse_multiline_frontmatter() {
    let content = "---\ntitle: Multi\ndescription: A long description\n---\nBody";
    let parts = MarkdownParts::parse(content);

    assert!(parts.has_frontmatter);
    assert_eq!(parts.body, "Body");
    assert_eq!(parts.frontmatter.get("title").and_then(|v| v.as_str()), Some("Multi"));
  }

  #[test]
  fn test_parse_empty_body() {
    let content = "---\ntitle: Hello\n---\n";
    let parts = MarkdownParts::parse(content);

    assert!(parts.has_frontmatter);
    assert_eq!(parts.body, "");
  }

  #[test]
  fn test_parse_no_content() {
    let content = "";
    let parts = MarkdownParts::parse(content);

    assert!(!parts.has_frontmatter);
    assert_eq!(parts.body, "");
  }

  #[test]
  fn test_get_field_exists() {
    let content = "---\ntitle: Hello\n---\nBody";
    let parts = MarkdownParts::parse(content);

    assert_eq!(parts.get_field("title").and_then(|v| v.as_str()), Some("Hello"));
  }

  #[test]
  fn test_get_field_not_exists() {
    let content = "---\ntitle: Hello\n---\nBody";
    let parts = MarkdownParts::parse(content);

    assert_eq!(parts.get_field("nonexistent"), None);
  }

  #[test]
  fn test_parse_yaml_numbers() {
    let content = "---\ncount: 42\nrating: 4.5\n---\nBody";
    let parts = MarkdownParts::parse(content);

    assert_eq!(
      parts.frontmatter.get("count").and_then(serde_json::Value::as_i64),
      Some(42)
    );
    assert_eq!(
      parts.frontmatter.get("rating").and_then(serde_json::Value::as_f64),
      Some(4.5)
    );
  }

  #[test]
  fn test_parse_yaml_booleans() {
    let content = "---\nenabled: true\ndisabled: false\n---\nBody";
    let parts = MarkdownParts::parse(content);

    assert_eq!(
      parts.frontmatter.get("enabled").and_then(serde_json::Value::as_bool),
      Some(true)
    );
    assert_eq!(
      parts.frontmatter.get("disabled").and_then(serde_json::Value::as_bool),
      Some(false)
    );
  }

  // DocumentKind tests
  #[test]
  fn test_classify_metadata_root() {
    let path = Path::new("metadata/characters/hero.md");
    assert_eq!(DocumentKind::classify_path(path), DocumentKind::Metadata);
  }

  #[test]
  fn test_classify_metadata_nested() {
    let path = Path::new("book-01/metadata/characters/hero.md");
    assert_eq!(DocumentKind::classify_path(path), DocumentKind::Metadata);
  }

  #[test]
  fn test_classify_metadata_deeply_nested() {
    let path = Path::new("series/book-01/part-01/metadata/scene.md");
    assert_eq!(DocumentKind::classify_path(path), DocumentKind::Metadata);
  }

  #[test]
  fn test_classify_article_root() {
    let path = Path::new("chapters/chapter-01.md");
    assert_eq!(DocumentKind::classify_path(path), DocumentKind::Article);
  }

  #[test]
  fn test_classify_article_nested() {
    let path = Path::new("book-01/chapters/chapter-01.md");
    assert_eq!(DocumentKind::classify_path(path), DocumentKind::Article);
  }

  #[test]
  fn test_classify_metadata_only_file() {
    let path = Path::new("metadata.md");
    assert_eq!(DocumentKind::classify_path(path), DocumentKind::Article);
  }

  #[test]
  fn test_classify_metadata_in_filename() {
    let path = Path::new("chapters/metadata-doc.md");
    assert_eq!(DocumentKind::classify_path(path), DocumentKind::Article);
  }

  #[test]
  fn test_classify_metadata_multiple_folders() {
    let path = Path::new("metadata/metadata/hero.md");
    assert_eq!(DocumentKind::classify_path(path), DocumentKind::Metadata);
  }

  #[test]
  fn test_markdown_parts_new() {
    let fm = json!({ "title": "Test" });
    let parts = MarkdownParts::new(fm, "Body".to_string(), true);

    assert!(parts.has_frontmatter);
    assert_eq!(parts.body, "Body");
  }

  // WorkspaceDocument tests

  #[test]
  fn test_workspace_document_metadata() {
    let metadata = MetadataEntity::new("test-id", "character", "global", json!({}), "Character body");
    let doc = WorkspaceDocument::Metadata(metadata.clone());

    assert_eq!(doc.kind(), DocumentKind::Metadata);
    assert!(doc.is_metadata());
    assert!(!doc.is_article());
    assert_eq!(doc.as_metadata(), Some(&metadata));
    assert_eq!(doc.as_article(), None);
  }

  #[test]
  fn test_workspace_document_article() {
    let article = ArticleDocument::new(json!({ "title": "Test Article" }), "Article body");
    let doc = WorkspaceDocument::Article(article.clone());

    assert_eq!(doc.kind(), DocumentKind::Article);
    assert!(!doc.is_metadata());
    assert!(doc.is_article());
    assert_eq!(doc.as_metadata(), None);
    assert_eq!(doc.as_article(), Some(&article));
  }

  #[test]
  fn test_workspace_document_metadata_kind() {
    let metadata = MetadataEntity::new("test", "character", "book-01", json!({}), "body");
    let doc = WorkspaceDocument::Metadata(metadata);

    match doc.kind() {
      DocumentKind::Metadata => {} // Expected
      DocumentKind::Article => panic!("Expected Metadata kind"),
    }
  }

  #[test]
  fn test_workspace_document_article_kind() {
    let article = ArticleDocument::new(json!({}), "body");
    let doc = WorkspaceDocument::Article(article);

    match doc.kind() {
      DocumentKind::Article => {} // Expected
      DocumentKind::Metadata => panic!("Expected Article kind"),
    }
  }

  #[test]
  fn test_workspace_document_as_metadata_unwrap() {
    let metadata = MetadataEntity::new("id", "scene", "ns", json!({ "name": "Scene" }), "body");
    let doc = WorkspaceDocument::Metadata(metadata.clone());

    let unwrapped = doc.as_metadata().unwrap();
    assert_eq!(unwrapped.id, "id");
    assert_eq!(unwrapped.type_, "scene");
  }

  #[test]
  fn test_workspace_document_as_article_unwrap() {
    let article = ArticleDocument::new(json!({ "title": "Chapter 1" }), "Once upon a time");
    let doc = WorkspaceDocument::Article(article.clone());

    let unwrapped = doc.as_article().unwrap();
    assert_eq!(unwrapped.title, Some("Chapter 1".to_string()));
    assert_eq!(unwrapped.body, "Once upon a time");
  }

  #[test]
  fn test_workspace_document_pattern_matching() {
    let metadata = MetadataEntity::new("meta", "note", "global", json!({}), "Note body");
    let article = ArticleDocument::new(json!({ "title": "Art" }), "Art body");

    let docs = vec![
      WorkspaceDocument::Metadata(metadata),
      WorkspaceDocument::Article(article),
    ];

    for doc in docs {
      match doc {
        WorkspaceDocument::Metadata(m) => {
          assert_eq!(m.id, "meta");
          assert_eq!(m.type_, "note");
        }
        WorkspaceDocument::Article(a) => {
          assert_eq!(a.title, Some("Art".to_string()));
        }
      }
    }
  }

  #[test]
  fn test_workspace_document_is_checks() {
    let metadata = MetadataEntity::new("m", "t", "n", json!({}), "b");
    let article = ArticleDocument::new(json!({}), "body");

    let meta_doc = WorkspaceDocument::Metadata(metadata);
    let art_doc = WorkspaceDocument::Article(article);

    assert!(meta_doc.is_metadata());
    assert!(!meta_doc.is_article());
    assert!(!art_doc.is_metadata());
    assert!(art_doc.is_article());
  }

  #[test]
  fn test_workspace_document_clone() {
    let metadata = MetadataEntity::new("id", "type", "ns", json!({ "field": "value" }), "body");
    let doc1 = WorkspaceDocument::Metadata(metadata);
    let doc2 = doc1.clone();

    assert_eq!(doc1.kind(), doc2.kind());
    assert_eq!(doc1.is_metadata(), doc2.is_metadata());
    assert!(doc1.as_metadata().is_some());
    assert_eq!(doc1.as_metadata(), doc2.as_metadata());
  }
}
