use tower_lsp::lsp_types::Position;

/// Convert LSP Position to byte offset in the document content.
/// Uses UTF-16 code units (LSP default encoding).
#[allow(dead_code)]
pub fn position_to_offset(content: &str, position: Position) -> Option<usize> {
  let Position { line, character } = position;

  // Handle empty content edge case
  if content.is_empty() {
    return if line == 0 && character == 0 { Some(0) } else { None };
  }

  let mut current_line = 0u32;
  let mut current_char_utf16 = 0u32;
  let mut byte_offset = 0usize;
  let mut prev_was_cr = false;

  for ch in content.chars() {
    // Handle CRLF: if we saw \r and now see \n, skip the \n
    // (it was already counted as part of the line break)
    if prev_was_cr && ch == '\n' {
      prev_was_cr = false;
      byte_offset += ch.len_utf8();
      continue;
    }
    prev_was_cr = false;

    // Check if we've reached the target line
    if current_line == line {
      // When on the target line, newline is a line boundary
      // Positions past the end of the line (before newline) are invalid
      if ch == '\r' || ch == '\n' {
        // Check if position is exactly at the newline boundary
        if current_char_utf16 == character {
          return Some(byte_offset);
        }
        // Position is past the end of this line - invalid
        return None;
      }

      // Check if we've reached the target character
      if current_char_utf16 == character {
        return Some(byte_offset);
      }

      // Count UTF-16 code units for this character (always 1 or 2)
      let utf16_units = u32::try_from(ch.encode_utf16(&mut [0; 2]).len()).expect("UTF-16 units <= 2");

      // Check if target character is beyond this position
      if current_char_utf16 + utf16_units > character {
        return None;
      }

      current_char_utf16 += utf16_units;
      byte_offset += ch.len_utf8();
    } else {
      // Still looking for the target line
      if ch == '\r' {
        prev_was_cr = true;
        current_line += 1;
        if current_line == line {
          current_char_utf16 = 0;
        }
      } else if ch == '\n' {
        current_line += 1;
        if current_line == line {
          current_char_utf16 = 0;
        }
      }
      byte_offset += ch.len_utf8();
    }
  }

  // Check if position is at the very end of content
  if current_line == line && current_char_utf16 == character {
    return Some(byte_offset);
  }

  None
}

/// Convert byte offset to LSP Position.
/// Uses UTF-16 code units (LSP default encoding).
pub fn offset_to_position(content: &str, offset: usize) -> Option<Position> {
  // Check if offset is within bounds
  if offset > content.len() {
    return None;
  }

  let mut current_offset = 0usize;
  let mut line = 0u32;
  let mut character = 0u32;
  let mut prev_was_cr = false;

  for ch in content.chars() {
    // Handle CRLF: if we saw \r and now see \n, skip the \n
    if prev_was_cr && ch == '\n' {
      prev_was_cr = false;
      current_offset += ch.len_utf8();
      continue;
    }
    prev_was_cr = false;

    if current_offset == offset {
      return Some(Position { line, character });
    }

    if ch == '\r' {
      prev_was_cr = true;
      line += 1;
      character = 0;
    } else if ch == '\n' {
      line += 1;
      character = 0;
    } else {
      // Count UTF-16 code units (always 1 or 2)
      let utf16_units = u32::try_from(ch.encode_utf16(&mut [0; 2]).len()).expect("UTF-16 units <= 2");
      character += utf16_units;
    }

    current_offset += ch.len_utf8();
  }

  // Handle offset at end of content
  if current_offset == offset {
    Some(Position { line, character })
  } else {
    None
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn position_to_offset_handles_ascii() {
    // ASCII: each char is 1 byte, 1 UTF-16 code unit
    let content = "Hello\nWorld";
    // Position at "W" (line 1, char 0)
    let pos = Position { line: 1, character: 0 };
    let offset = position_to_offset(content, pos);
    assert_eq!(offset, Some(6)); // After "Hello\n"
  }

  #[test]
  fn position_to_offset_handles_cjk() {
    // CJK characters are 3 bytes in UTF-8, but 1 UTF-16 code unit
    let content = "Hello\n世界";
    // Position at "世" (line 1, char 0)
    let pos = Position { line: 1, character: 0 };
    let offset = position_to_offset(content, pos);
    assert_eq!(offset, Some(6)); // After "Hello\n"

    // Position at "界" (line 1, char 1)
    let pos = Position { line: 1, character: 1 };
    let offset = position_to_offset(content, pos);
    assert_eq!(offset, Some(9)); // After "Hello\n世" (3 bytes for 世)
  }

  #[test]
  fn position_to_offset_handles_emoji_utf16() {
    // Emoji like 😀 are surrogate pairs: 2 UTF-16 code units
    let content = "Hello\n😀World";
    // Position at start of emoji (line 1, char 0)
    let pos = Position { line: 1, character: 0 };
    let offset = position_to_offset(content, pos);
    assert_eq!(offset, Some(6)); // After "Hello\n"

    // Position at 'W' after emoji (emoji is 2 UTF-16 units)
    let pos = Position { line: 1, character: 2 };
    let offset = position_to_offset(content, pos);
    assert_eq!(offset, Some(10)); // After "Hello\n😀" (4 bytes for emoji)
  }

  #[test]
  fn position_to_offset_rejects_out_of_range() {
    let content = "Hello\nWorld";
    // Line out of range
    let pos = Position { line: 10, character: 0 };
    assert_eq!(position_to_offset(content, pos), None);

    // Character out of range
    let pos = Position {
      line: 0,
      character: 100,
    };
    assert_eq!(position_to_offset(content, pos), None);
  }

  #[test]
  fn position_to_offset_rejects_line_boundary_overflow() {
    // Character past end of line with another line following should be rejected
    // "Hello\nWorld" - line 0 has "Hello" (5 chars), line 1 has "World"
    let content = "Hello\nWorld";

    // Position { line: 0, character: 5 } is at the newline boundary - valid
    let pos = Position { line: 0, character: 5 };
    assert_eq!(position_to_offset(content, pos), Some(5));

    // Position { line: 0, character: 6 } is past end of line 0 - should be rejected
    let pos = Position { line: 0, character: 6 };
    assert_eq!(position_to_offset(content, pos), None);

    // Position at start of line 1 should work
    let pos = Position { line: 1, character: 0 };
    assert_eq!(position_to_offset(content, pos), Some(6));
  }

  #[test]
  fn offset_to_position_handles_ascii() {
    // ASCII: each char is 1 byte, 1 UTF-16 code unit
    let content = "Hello\nWorld";
    // Offset at 'W' (position 6)
    let pos = offset_to_position(content, 6);
    assert_eq!(pos, Some(Position { line: 1, character: 0 }));
  }

  #[test]
  fn offset_to_position_handles_cjk() {
    // CJK characters are 3 bytes in UTF-8, but 1 UTF-16 code unit
    let content = "Hello\n世界";
    // Offset at "世" (position 6)
    let pos = offset_to_position(content, 6);
    assert_eq!(pos, Some(Position { line: 1, character: 0 }));

    // Offset at "界" (position 9)
    let pos = offset_to_position(content, 9);
    assert_eq!(pos, Some(Position { line: 1, character: 1 }));
  }

  #[test]
  fn offset_to_position_handles_emoji_utf16() {
    // Emoji like 😀 are surrogate pairs: 2 UTF-16 code units
    let content = "Hello\n😀World";
    // Offset at start of emoji (position 6)
    let pos = offset_to_position(content, 6);
    assert_eq!(pos, Some(Position { line: 1, character: 0 }));

    // Offset after emoji (position 10), should be at 'W'
    // Emoji is 2 UTF-16 code units
    let pos = offset_to_position(content, 10);
    assert_eq!(pos, Some(Position { line: 1, character: 2 }));
  }

  #[test]
  fn offset_to_position_rejects_out_of_range() {
    let content = "Hello\nWorld";
    // Offset beyond content length
    assert_eq!(offset_to_position(content, 1000), None);
  }

  #[test]
  fn position_to_offset_handles_crlf() {
    // CRLF: \r\n should be treated as single line break
    let content = "Hello\r\nWorld";

    // Position at 'o' (line 0, char 4)
    let pos = Position { line: 0, character: 4 };
    assert_eq!(position_to_offset(content, pos), Some(4));

    // Position at end of "Hello" (line 0, char 5) - at \r
    let pos = Position { line: 0, character: 5 };
    assert_eq!(position_to_offset(content, pos), Some(5));

    // Position past end of line 0 (char 6) - invalid
    let pos = Position { line: 0, character: 6 };
    assert_eq!(position_to_offset(content, pos), None);

    // Position at start of line 1 (char 0) - at 'W'
    let pos = Position { line: 1, character: 0 };
    assert_eq!(position_to_offset(content, pos), Some(7)); // After \r\n

    // Position at 'W' (line 1, char 0)
    let pos = Position { line: 1, character: 0 };
    assert_eq!(position_to_offset(content, pos), Some(7));
  }

  #[test]
  fn position_to_offset_handles_cr_only() {
    // Old Mac style: \r alone is line break
    let content = "Hello\rWorld";

    // Position at end of "Hello" (line 0, char 5) - at \r
    let pos = Position { line: 0, character: 5 };
    assert_eq!(position_to_offset(content, pos), Some(5));

    // Position at start of line 1 (char 0) - at 'W'
    let pos = Position { line: 1, character: 0 };
    assert_eq!(position_to_offset(content, pos), Some(6)); // After \r
  }

  #[test]
  fn offset_to_position_handles_crlf() {
    // CRLF: \r\n should be treated as single line break
    let content = "Hello\r\nWorld";

    // Offset at 'W' (position 7)
    let pos = offset_to_position(content, 7);
    assert_eq!(pos, Some(Position { line: 1, character: 0 }));

    // Offset at \r (position 5)
    let pos = offset_to_position(content, 5);
    assert_eq!(pos, Some(Position { line: 0, character: 5 }));

    // Offset after \r\n (position 7)
    let pos = offset_to_position(content, 7);
    assert_eq!(pos, Some(Position { line: 1, character: 0 }));

    // Offset at end of content
    let pos = offset_to_position(content, 12);
    assert_eq!(pos, Some(Position { line: 1, character: 5 }));
  }

  #[test]
  fn offset_to_position_handles_cr_only() {
    // Old Mac style: \r alone is line break
    let content = "Hello\rWorld";

    // Offset at \r (position 5)
    let pos = offset_to_position(content, 5);
    assert_eq!(pos, Some(Position { line: 0, character: 5 }));

    // Offset at 'W' (position 6)
    let pos = offset_to_position(content, 6);
    assert_eq!(pos, Some(Position { line: 1, character: 0 }));
  }

  #[test]
  fn position_to_offset_handles_crlf_with_cjk() {
    // CRLF with CJK characters (3 bytes UTF-8, 1 UTF-16 unit)
    let content = "Hello\r\n世界";

    // Position at "世" (line 1, char 0)
    let pos = Position { line: 1, character: 0 };
    assert_eq!(position_to_offset(content, pos), Some(7)); // After \r\n

    // Position at "界" (line 1, char 1)
    let pos = Position { line: 1, character: 1 };
    assert_eq!(position_to_offset(content, pos), Some(10)); // After \r\n世
  }

  #[test]
  fn offset_to_position_handles_crlf_with_emoji() {
    // CRLF with emoji (4 bytes UTF-8, 2 UTF-16 units)
    let content = "Hello\r\n😀World";

    // Offset at emoji (position 7)
    let pos = offset_to_position(content, 7);
    assert_eq!(pos, Some(Position { line: 1, character: 0 }));

    // Offset after emoji (position 11)
    let pos = offset_to_position(content, 11);
    assert_eq!(pos, Some(Position { line: 1, character: 2 }));
  }
}
