---
name: lint-markdown
description: Check and fix formatting and other issues in markdown files using markdownlint-cli2
---

# Markdown linting

Check markdown files for common mistakes.

## Installation and usage

Run `npm install` from the repository root to install `markdownlint-cli2` from `devDependencies` in the root `package.json`. Then run `npx markdownlint-cli2 <command>` to use the locally installed version.

## Configuration

The markdownlint-cli2 configuration is in `.markdownlint-cli2.yaml` at the repository root.

The configuration has the following structure:

- `globs`: file patterns to lint (currently `**/*.md`), so no command-line arguments are needed.
- `ignores`: paths excluded from linting.
- `config`: markdownlint rule overrides (e.g., `first-line-h1`, `line-length`, `ol-prefix`).

## Check Markdown

Run `npx markdownlint-cli2` to lint Markdown files according to the configuration.

## Fix issues

Run with the `--fix` flag to automatically fix supported issues:

```bash
npx markdownlint-cli2 --fix
```

## Testing

Run the same lint command again to verify all issues are fixed. There should be no errors reported.
