# gj

🪵 Dead simple CLI for journaling.  
Write logs from the terminal, stored in Notion.

## Install

```bash
brew tap efekrskl/gj
brew install gj
```

## Usage

```bash
gj "Wrote some code"
gj "Workout; Dinner with friends"
```

- Entries are timestamped and grouped by month
- Use `;` to add multiple entries at once

## Setup

First-time only:

```bash
gj --setup
```

- Prompts for Notion token and database ID
- Saves config to `~/.gj/config.json`

## Notion

1. [Create an integration](https://www.notion.so/my-integrations)
2. Create a page in your workspace
3. Copy the integration token and page ID
4. Run `gj --setup`

## License

MIT
