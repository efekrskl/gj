# gj

🪵 gj is a dead simple journaling CLI.
Type your thoughts into the terminal — they get logged to Notion, one page per day. No clutter, no fuss.

<img width="1293" alt="image" src="https://github.com/user-attachments/assets/240aec31-1cc4-4e66-9b4c-3ca75429efa3" />

## Install

### MacOS (homebrew)

```bash
brew tap efekrskl/gj
brew install gj
```

### Arch Linux (build from source)

_Note:_ It will probably work on other linux distributions too. But I haven't personally tested it.

```bash
git clone https://github.com/efekrskl/gj && cd gj
cargo build --release
cp target/release/gj /usr/bin/
```

### Arch Linux (from AUR)

You can install AUR packages either via `paru` or `yay`. The procedure is the same for both.

```bash
paru -S gj-git
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
