# gj

🪵 gj is a dead simple journaling CLI.
Type your thoughts into the terminal — they get logged to Notion, one page per day. No clutter, no fuss.

<img width="1293" alt="image" src="https://github.com/user-attachments/assets/240aec31-1cc4-4e66-9b4c-3ca75429efa3" />


## Install

```bash
brew tap efekrskl/gj
brew install gj
```

## Usage

```bash
gj "wrote some code"
```

- Entries are timestamped and grouped by date
- Use ; to log multiple entries in one line

```bash
gj "pair programmed on a new feature; wrote some tests" --tags="pair programming, tests"
```

- Use --tags to add tags to the current day's page (stored as a Notion multi-select property)

## Setup

First-time only:

1. [Create a Notion integration](https://www.notion.so/my-integrations)
2. Create a page in your workspace
3. In the meatballs menu (⋯) → Connections, connect your integration
4. Copy the integration token and page ID
5. Run `gj --setup` and paste the token and page ID when prompted

## License

MIT
