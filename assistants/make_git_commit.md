This document outlines the process for creating a standardized git commit.

### Objective
To create a git commit with a clear and informative commit message, explaining the "what" and "why" of the changes. This process assumes that the relevant changes have already been staged.

### Steps

1.  **Understand the Changes:**
    *   To understand what has changed, I will first look at the staged files. I can do this by running `git diff --staged`. This will give me the context I need to write an accurate commit message.

2.  **Draft the Commit Message:**
    *   I will structure the commit message according to the following format:

        ```
        <type>: <subject>

        <body>
        ```

    *   **Type:** The type of change being committed. This helps to quickly understand the nature of the commit.
        *   `feat`: A new feature.
        *   `fix`: A bug fix.
        *   `docs`: Changes to documentation.
        *   `style`: Formatting, missing semi-colons, etc.; no production code change.
        *   `refactor`: Refactoring production code, e.g., renaming a variable.
        *   `test`: Adding missing tests, refactoring tests; no production code change.
        *   `chore`: Updating build tasks, package manager configs, etc.; no production code change.
        *   `tooling`: Changes to AI assistants and corresponding tooling.

    *   **Subject:** A short, imperative summary of the change.
        *   It should be 50 characters or less.
        *   It should not end with a period.

    *   **Body:** A more detailed explanation of the changes.
        *   This section should explain the "why" behind the change. What was the problem, and how does this commit solve it?
        *   It should be roughly 4 lines
        *   It should be wrapped at 72 characters.
        *   DO NOT imply things or DO NOT invent benefits
        *   Stick strictly to facts

3.  **Create the Commit:**
    *   Once the message is ready, I will execute the commit using the following command:
        ```bash
        git commit -m "<type>: <subject>" -m "<body>"
        ```
