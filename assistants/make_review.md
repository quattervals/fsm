# Make a Code Review

Review the code and make suggestions on how to improve it

## Usage
Provide parameters in natural language in your task message. Examples:
- "Make a code review"
- "Make a code review and focus on xy"
- "Make code review [focus on xy] and write things to file (e.g. code_review.md)"

## Form
- Structure your comments: quality, coding guideline, what if, unit tests,

## Things to Consider
- Point out what could go wrong if the tool/library/module was used differently than in the current code
- Make suggestions about missing cases in the unit tests
- Call out abuse of patterns, types, concepts, crates
- Take the perspective of the users
- Point out where the documentation could be improved
  - but don't put docstrings for the sake of docstrings
- Check if the documentation matches the actual implementation
  - on high level documentation
  - on function level
- Check if the docstrings match the implementation

API:
- only if there actually are APIs for consumers
- are the features orthogonal
- what if some feature was abused be it intentionally or accidentially
- what if users use the API in an unintended way


## Guidelines
- Tone: BE CONCISE, prefer being LACONIC instead of being a bullshitter
- Be truthful, stay with the facts
- Ask if things are unclear
