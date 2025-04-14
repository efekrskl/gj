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
gj "wrote some code"
```

- Entries are timestamped and grouped by month
- Use `;` to add multiple entries at once

```bash
gj "implemented the very first version of gj; documented my learnings with gj; solved some bugs"
```

Will be shown in Notion as:

<img width="1723" alt="image" src="https://github.com/user-attachments/assets/b7726166-5648-4870-9192-aa64aaadf24e" />


## Setup

First-time only:

1. [Create an integration](https://www.notion.so/my-integrations)
2. Create a page in your workspace
3. Copy the integration token and page ID
4. Run `gj --setup`

## License

MIT
