---
name: maintain-docs
description: Maintain technical and AI assistant documentation following structured conventions. Use when editing AGENTS.md, CLAUDE.md, SKILL.md, .cursorrules, or any project documentation that provides context to AI assistants and developers.
license: MIT
compatibility: opencode
metadata:
  author: zerozawa
  version: '1.0.0'
  repository: https://github.com/novelsaga/NSaga
---

# Documentation Maintenance Guide

Maintain project documentation with consistent structure and quality.

---

## Quick Start

| Task             | Approach                                                   |
| ---------------- | ---------------------------------------------------------- |
| Add new rule     | Follow "Golden Rules → Details → WHY" pattern              |
| Update checklist | Keep it at the end, use checkboxes                         |
| Add workflow     | Use step-by-step with commands + WHY                       |
| Fix structure    | Ensure "Opening Points → Details → Closing Reminders" flow |

---

## Document Structure (Three-Part Flow)

All project docs must follow this flow:

### 1. Opening Points (Golden Rules)

- **Golden Rules** — Numbered list of hard rules
- Each rule is a single sentence with **bold** keywords
- Include "WHY" inline or in a following paragraph

### 2. Middle Details (Deep Dive)

- Group by topic with `##` headers
- Use subsections (`###`) for specific areas
- Include code examples, tables, and commands
- Every section ends with `_WHY: Explanation..._`

### 3. Closing Reminders (Summary)

- **Summary Checklist** — Checkbox list for verification
- **Key Facts** — Bullet points of essential info
- Keep it brief but comprehensive

---

## Quick Start Format

Place a Quick Start table immediately after the introduction, before Golden Rules:

```markdown
## Quick Start

| Task          | Approach     |
| ------------- | ------------ |
| Common task 1 | How to do it |
| Common task 2 | How to do it |
| Common task 3 | How to do it |
```

**Requirements:**

- Use table format with Task and Approach columns
- List 3-5 most common actions readers will want to do
- Keep each cell concise (one line preferred)
- Link to detailed sections when applicable

---

## Golden Rules Format

```markdown
## Golden Rules

1. **Rule one** — Description with **bold** keywords.
2. **Rule two** — Another important rule.
3. **Rule three** — Third essential guideline.

_WHY: Brief explanation of why these rules exist and what problems they prevent._
```

**Requirements:**

- Use `1.` numbering (not `-` bullets)
- Keywords in **bold**
- Em-dash (`—`) after rule title
- WHY paragraph follows immediately

---

## Section Structure

Each major section follows this pattern:

```markdown
## Section Name

Brief intro sentence.

### Subsection (if needed)

Content here: code blocks, tables, lists.

_WHY: Why this convention matters._

---
```

**Required elements:**

- `##` header for main sections
- `###` for subsections
- `_WHY: ..._` italic paragraph at end
- `---` separator between sections

---

## WHY Paragraph Format

All WHY paragraphs use this exact format:

```markdown
_WHY: Single sentence explaining the rationale. Can include multiple clauses separated by semicolons. References to external standards or links are optional._
```

**Requirements:**

- Must start with `_WHY:`
- Must end with `_`
- Single paragraph (no line breaks)
- Italic formatting

---

## Code Examples

Use fenced code blocks with language tags:

````markdown
   ```rust
   // Example code here
   ```
````

````bash
   # Commands here
   ```

   ```

````

**For commands:**

- Include comments explaining what it does
- Group related commands together
- Use realistic examples from the actual project

---

## Tables

Use tables for comparisons and references:

```markdown
| Column A | Column B | Column C |
|----------|----------|----------|
| Value 1  | Value 2  | Value 3  |
````

**When to use:**

- Naming conventions
- Command references
- Comparison matrices

---

## Summary Checklist Format

Always end with:

```markdown
## Summary Checklist

Before [action], verify:

- [ ] Item one
- [ ] Item two
- [ ] Item three

Key facts:

- **Fact**: Value
- **Fact**: Value
```

**Requirements:**

- Use `- [ ]` checkbox format
- Group related items logically
- End with "Key facts" bullet list
- Use **bold** for fact labels

---

## Common Mistakes to Avoid

1. **Wrong WHY format**
   - ❌ `### Why This Matters` + bullet points
   - ✅ `_WHY: Single paragraph explanation._`

2. **Missing section separators**
   - ❌ Sections run together without `---`
   - ✅ Every section ends with `---`

3. **Checklist in wrong place**
   - ❌ Checklist embedded in middle sections
   - ✅ Checklist only in final Summary Checklist

4. **Inconsistent rule formatting**
   - ❌ Mixed bullet styles or missing bold keywords
   - ✅ All rules: `1. **Keyword** — Description.`

5. **No WHY after sections**
   - ❌ Section ends without explanation
   - ✅ Every section has `_WHY: ..._`

6. **Using non-English languages**
   - ❌ Mixing Chinese or other languages in documentation
   - ✅ Write all documentation in English only

---

## References

- AGENTS.md — Project knowledge base and conventions
- CLAUDE.md — Claude Code specific instructions
- SKILL.md — OpenCode skill definitions
- .cursorrules — Cursor IDE rules and preferences
- Any AI assistant context files — Follow the same three-part structure
