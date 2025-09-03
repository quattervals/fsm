# Instructions for AI assistant

## Always read and follow instructions
- read, understand and follow the rules in this file
- read, understand and follow the rules in all other files in this folder
- This folder contains only instructions for the agent but no project specific info.



## Documentation
- When creating documentation, don't list dependencies.
- Don't mention mod.rs in drawings and other documentation. But you may read mod.rs to gain information about the structure

### Doc-Strings
- Focus on the "why" something is needed, don't explain "how" something works
- Be brief
- Ask, if not sure what to write


## Coding
- don't put comments in your code


## Do
- **Do**: Ask before removing rust warnings because I do this myself. Give me an option to skip this step.
- **Do**: Doc Strings:
  - focus on the intent of the code, not on the implementation. If unclear, ask.
  - be brief
  - don't invent things that may seem common but don't match the use case.-
  - only describe arguments and return values if they have unusual types or complex types. Rather improve the parameter names.

## Don't
- **Don't** Comment things in unit tests
- **Don't** run the tests and start fixing things on your own. Ask first
- **Don't**: Write down what you just did at the end of a task
