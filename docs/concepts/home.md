# Home

The home directory structure that custodex uses at runtime.

## Layout

```
.local/home/
├── .custodex/               # persistent state (like ~/.claude/)
│   └── ...                  # structure determined by each bot template
│
├── code-agent/              # working directory for the code-agent template
├── simple-chat/             # working directory for the simple-chat template
└── <template-name>/         # working directory for any template
```

## .local/home/

The root home directory. All bot actions are confined within this directory. In development, this lives at `.local/home/` relative to the project root (gitignored).

Each bot template is assigned a working directory at `.local/home/<template-name>/`. This is where the bot does its work — file reads, writes, command execution all happen here.

## .local/home/.custodex/

Persistent state directory. Analogous to `~/.claude/` for Claude Code.

Stores:
- Saved bot instance configurations
- Conversation history
- Cached data
- Any other state that should survive restarts

The internal structure of `.custodex/` is **not prescribed by the framework**. Each bot template decides how to organize its own state within this directory.

## Persistence model

Unlike Claude sessions which are ephemeral, custodex bot instances are designed to be **persistent**:

- On first launch: `.custodex/` is empty, user creates instances via the dashboard wizard
- On subsequent launches: custodex restores previously created instances from `.custodex/`
- While running: instances persist state continuously to `.custodex/`

This means bots maintain their conversation history and configuration across restarts.

## Working directories

Each bot template gets `.local/home/<template-name>/` as its working directory (`cwd`). The bot operates within this directory — the Landlock sandbox can restrict file access to this path.

Templates own their working directory entirely. The framework creates the directory on startup but does not manage its contents.
