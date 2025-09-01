# Release Notes for Developers Generation Tool

## Purpose
Generate comprehensive release notes between two git tags by analyzing git history and focusing on the reasoning behind changes. This tool emphasizes the "why" behind changes rather than just listing what changed, following the project's documentation philosophy.

## Usage
Provide parameters in natural language in your task message. Examples:

- "Generate release notes from v0.1.0 to HEAD, output to RELEASE_NOTES.md"
- "Create release notes between v0.1.0 and v0.2.0"
- "Make release notes from last tag to current, save as release-notes-v0.2.0.md"
- "Generate release notes from v0.1.0 to v0.2.0, output to docs/CHANGELOG.md"


## Parameter Extraction
I will parse your message to identify:

- **from_tag**: Starting git tag or commit reference
- **to_tag**: Ending git tag, commit reference, or "HEAD" for unreleased changes
- **output_file**: Destination file path (defaults to "RELEASE_NOTES.md" if not specified)

## Analysis Process

### 1. Git History Analysis
- Use `git --no-pager log <from_tag>..<to_tag>` to get commit list
- Use `git --no-pager log --stat <from_tag>..<to_tag>` to understand file changes
- Use `git --no-pager show <commit>` for detailed analysis of significant changes

**Important**: Always use `--no-pager` flag with git commands to prevent them from opening in a pager (like `less`) which can cause the terminal to hang waiting for user input.

### 2. Change Categorization
Analyze commits and categorize them into:

- **🚀 Features**: New functionality or capabilities
- **🐛 Bug Fixes**: Corrections to existing functionality
- **💥 Breaking Changes**: Changes that break backward compatibility
- **📚 Documentation**: Updates to documentation, comments, or examples
- **🔧 Internal**: Refactoring, code organization, or internal improvements
- **⚡ Performance**: Optimizations and performance improvements
- **🧪 Testing**: Test additions, improvements, or fixes

### 3. Reasoning Analysis
For each change, focus on:

- **Why was this change needed?** (motivation, problem being solved)
- **What impact does it have?** (user-facing benefits, technical improvements)
- **What was the context?** (related issues, architectural decisions)

### 4. Content Sources
Extract information from:

- Commit messages (especially detailed ones)
- File diffs to understand scope of changes
- Code comments and documentation changes
- Patterns in related commits

## Output Format

Generate structured markdown following this template:

```markdown
# Release Notes: [from_tag] → [to_tag]

## Overview
Brief summary of the release, highlighting major themes and improvements.

## 🚀 Features
### Feature Name
Brief description that naturally integrates the motivation, problem solved, and user benefits. Keep it concise and focus on what matters to users.

## 🐛 Bug Fixes
### Bug Description
Concise explanation of what was fixed and why it improves the user experience.

## 💥 Breaking Changes
### Change Description
Brief explanation of the change, why it was necessary, and how users should adapt.

## 📚 Documentation
### Documentation Updates
Brief description of documentation improvements and what they clarify.

## 🔧 Internal Improvements
### Improvement Description
Concise explanation of technical improvements and their benefits for maintainability.

## ⚡ Performance
### Performance Improvement
Brief description of optimizations and their impact.

## 🧪 Testing
### Testing Improvements
Concise description of test additions and improvements.
```

## Guidelines

### Focus on "Why"
- Emphasize the reasoning and motivation behind changes
- Explain the problem being solved, not just the solution
- Connect changes to user benefits or technical improvements

### Be Concise but Meaningful
- Keep descriptions brief but informative
- Avoid technical jargon unless necessary
- Focus on impact and value

### Writing Style
- Integrate reasoning naturally into descriptions without explicit "Why:", "Impact:", "Details:" labels
- Write concise but meaningful descriptions that explain both what changed and why it matters
- Focus on user-facing benefits and technical improvements that matter to developers using the framework
- Tone: BE CONCISE, prefer being LACONIC instead of being a bullshitter
- Be truthful. Stay with the facts and don't invent flowry things like
  -  'complete'
  -  'best'
  -  'advanced'

### Content Filtering
- Skip development infrastructure changes (CI/CD, project management tools, etc.)
- Exclude contributor lists and generation timestamps
- Don't reference specific commit hashes or commit messages
- Prioritize user-facing changes over internal refactoring
- Skip trivial commits (typo fixes, minor formatting changes)
- Skip `.vscode` related changes

### Quality Standards
- Group related changes together under coherent feature/improvement descriptions
- Combine multiple related commits into single, meaningful entries
- Focus on the value and impact rather than technical implementation details
