# New Instance Wizard

How a user creates a new bot instance from the dashboard.

## Flow

```
[+ New Instance] → Template Selection → Config Step 1 → ... → Config Step N → Launch
```

## Step 1: Template Selection

A list of available bot templates. Each shows:
- Template name
- Short description
- Number of config steps

User picks one and clicks Next.

```
┌─────────────────────────────────┐
│     Select Bot Template         │
│                                 │
│  ○ code-agent                   │
│    Full code agent with tool    │
│    execution and approval flow  │
│                                 │
│  ○ simple-chat                  │
│    Text conversation only       │
│                                 │
│  ○ auto-approve                 │
│    Code agent, auto-approves    │
│    all tool calls               │
│                                 │
│              [Cancel]  [Next →] │
└─────────────────────────────────┘
```

## Steps 2..N: Template-Specific Config

Each template defines its own wizard steps. The dashboard calls the config's render method for each step. The template controls:
- What fields are shown
- Validation logic (Next is disabled until step is valid)
- How many steps there are

Example for `code-agent`:

**Step 2: Working Directory**
```
┌─────────────────────────────────┐
│  Working Directory              │
│                                 │
│  Path: [/home/user/project    ] │
│                                 │
│  [← Back]  [Cancel]  [Next →]  │
└─────────────────────────────────┘
```

**Step 3: Model**
```
┌─────────────────────────────────┐
│  Model Selection                │
│                                 │
│  Model: [o4-mini           ▾]   │
│                                 │
│  [← Back]  [Cancel]  [Next →]  │
└─────────────────────────────────┘
```

**Step 4: Approval Policy**
```
┌─────────────────────────────────┐
│  Approval Policy                │
│                                 │
│  ○ Ask for approval (default)   │
│  ○ Auto-approve all             │
│  ○ Deny all (read-only)         │
│                                 │
│  [← Back]  [Cancel]  [Launch]   │
└─────────────────────────────────┘
```

## After Launch

The wizard closes. The new instance appears in the Instances panel in "initializing" state. Once the CodexThread is ready, it transitions to "ready" and begins accepting messages from its bound Telegram chat.

## Implementation

The dashboard drives the wizard generically:

1. Collect registered templates
2. Render template selection
3. For the chosen template, iterate through `template.wizard_steps()`
4. Each step: call `config.render_step(ui, step_index)` and `config.validate_step(step_index)`
5. On final step: call `template.launch(config)` to create the instance
