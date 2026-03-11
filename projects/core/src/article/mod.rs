use serde_json::Value;

use crate::document::MarkdownParts;

/// Legacy Article type - kept for backward compatibility
#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Article {
  content: String,
}

impl Article {
  /// Create Article from string
  #[must_use]
  pub fn new(content: impl Into<String>) -> Self {
    Self {
      content: content.into(),
    }
  }

  /// Get content reference
  #[must_use]
  pub fn content_ref(&self) -> &str {
    &self.content
  }
}

/// Document with structured metadata extracted from markdown
///
/// `ArticleDocument` represents a parsed markdown document with separated frontmatter
/// metadata and body content. It provides convenient access to common metadata fields
/// like title, `chapter_number`, and order.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ArticleDocument {
  /// Parsed frontmatter as JSON value
  pub frontmatter: Value,
  /// Markdown body content
  pub body: String,
  /// Article title (extracted from frontmatter)
  pub title: Option<String>,
  /// Chapter number (extracted from frontmatter)
  pub chapter_number: Option<u32>,
  /// Article order/sequence number (extracted from frontmatter)
  pub order: Option<i32>,
  /// Word count of the body
  pub word_count: u32,
}

impl ArticleDocument {
  /// Create `ArticleDocument` from frontmatter and body
  ///
  /// # Arguments
  /// * `frontmatter` - Parsed frontmatter as JSON value
  /// * `body` - Markdown body content
  ///
  /// # Returns
  /// New `ArticleDocument` with extracted metadata and computed word count
  #[must_use]
  pub fn new(frontmatter: Value, body: impl Into<String>) -> Self {
    let body = body.into();
    let word_count = u32::try_from(body.split_whitespace().count()).unwrap_or(u32::MAX);

    let title = frontmatter
      .get("title")
      .and_then(|v| v.as_str())
      .map(ToString::to_string);

    let chapter_number = frontmatter
      .get("chapter_number")
      .and_then(serde_json::Value::as_u64)
      .map(|n| u32::try_from(n).unwrap_or(u32::MAX));

    let order = frontmatter
      .get("order")
      .and_then(serde_json::Value::as_i64)
      .map(|n| i32::try_from(n).unwrap_or(i32::MAX));

    Self {
      frontmatter,
      body,
      title,
      chapter_number,
      order,
      word_count,
    }
  }

  /// Create `ArticleDocument` from `MarkdownParts`
  ///
  /// # Arguments
  /// * `parts` - Parsed markdown with separated frontmatter and body
  ///
  /// # Returns
  /// New `ArticleDocument` with extracted metadata from parts
  #[must_use]
  pub fn from_parts(parts: MarkdownParts) -> Self {
    Self::new(parts.frontmatter, parts.body)
  }
}

impl From<MarkdownParts> for ArticleDocument {
  fn from(parts: MarkdownParts) -> Self {
    Self::from_parts(parts)
  }
}

#[cfg(test)]
mod tests {
  use serde_json::json;

  use super::*;

  // ArticleDocument tests

  #[test]
  fn test_article_document_new_minimal() {
    let frontmatter = json!({});
    let doc = ArticleDocument::new(frontmatter, "hello world");

    assert_eq!(doc.body, "hello world");
    assert_eq!(doc.word_count, 2);
    assert_eq!(doc.title, None);
    assert_eq!(doc.chapter_number, None);
    assert_eq!(doc.order, None);
  }

  #[test]
  fn test_article_document_new_with_title() {
    let frontmatter = json!({
      "title": "My Article"
    });
    let doc = ArticleDocument::new(frontmatter, "some content");

    assert_eq!(doc.title, Some("My Article".to_string()));
    assert_eq!(doc.word_count, 2);
  }

  #[test]
  fn test_article_document_new_with_all_metadata() {
    let frontmatter = json!({
      "title": "Chapter 1",
      "chapter_number": 1,
      "order": 10
    });
    let doc = ArticleDocument::new(frontmatter, "once upon a time there was a story");

    assert_eq!(doc.title, Some("Chapter 1".to_string()));
    assert_eq!(doc.chapter_number, Some(1));
    assert_eq!(doc.order, Some(10));
    assert_eq!(doc.word_count, 8);
  }

  #[test]
  fn test_article_document_word_count_empty() {
    let frontmatter = json!({});
    let doc = ArticleDocument::new(frontmatter, "");

    assert_eq!(doc.word_count, 0);
  }

  #[test]
  fn test_article_document_word_count_whitespace() {
    let frontmatter = json!({});
    let doc = ArticleDocument::new(frontmatter, "   word1   \n\n   word2  ");

    assert_eq!(doc.word_count, 2);
  }

  #[test]
  fn test_article_document_from_parts() {
    let parts = MarkdownParts::new(
      json!({
        "title": "Test",
        "chapter_number": 5,
        "order": 20
      }),
      "test body content".to_string(),
      true,
    );

    let doc = ArticleDocument::from_parts(parts);

    assert_eq!(doc.title, Some("Test".to_string()));
    assert_eq!(doc.chapter_number, Some(5));
    assert_eq!(doc.order, Some(20));
    assert_eq!(doc.body, "test body content");
    assert_eq!(doc.word_count, 3);
  }

  #[test]
  fn test_article_document_from_markdown_parts() {
    let parts = MarkdownParts::new(
      json!({
        "title": "From Parts",
        "order": 1
      }),
      "markdown body".to_string(),
      true,
    );

    let doc: ArticleDocument = parts.into();

    assert_eq!(doc.title, Some("From Parts".to_string()));
    assert_eq!(doc.order, Some(1));
    assert_eq!(doc.body, "markdown body");
  }

  #[test]
  fn test_article_document_invalid_metadata_types() {
    let frontmatter = json!({
      "title": 123,
      "chapter_number": "not a number",
      "order": "also not a number"
    });
    let doc = ArticleDocument::new(frontmatter, "body");

    // Invalid types should result in None
    assert_eq!(doc.title, None);
    assert_eq!(doc.chapter_number, None);
    assert_eq!(doc.order, None);
  }

  #[test]
  fn test_article_document_preserves_frontmatter() {
    let frontmatter = json!({
      "title": "Test",
      "author": "Jane",
      "tags": ["rust", "article"]
    });
    let doc = ArticleDocument::new(frontmatter.clone(), "body");

    // Frontmatter should be preserved
    assert_eq!(doc.frontmatter, frontmatter);
  }

  // Legacy Article tests

  #[test]
  fn test_article_new() {
    let article = Article::new("test content");
    assert_eq!(article.content_ref(), "test content");
  }

  #[test]
  fn test_article_from_string() {
    let content = "article body".to_string();
    let article = Article::new(content);
    assert_eq!(article.content_ref(), "article body");
  }
}
