# dectl — Reviewer Guide

This guide is for an independent, hands-on review of dectl's agent-led project workflow. Please report what you observe, including friction or failures; no particular verdict is expected.

## What You Will Review

The demo follows one small project from a developer brief to reviewed implementation:

1. The AI agent completes a project specification from a brief and asks clarifying questions.
2. The agent creates specs using the project's SDD skill, templates, examples, and rules.
3. The developer reviews and approves the specs and task list.
4. The agent implements one approved task through dectl's task workflow.
5. Build, tests, lint, review output, task state, and a local preview make the result inspectable.

The developer supplies information and approves decisions. The agent drafts specs and code. dectl prepares project context, coordinates workflow steps, runs configured checks, and records workflow artifacts.

## Requirements

- `dectl` installed and available on `PATH`
- Node.js 18 or newer and npm
- Network access to the npm registry for the sample app dependencies
- An AI coding environment that can read and edit workspace files and run terminal commands

## Prepare the Demo

Use a new, disposable folder. The script creates the project beside itself, so do not run it from a real project or a directory with existing demo files.

```bash
mkdir -p ~/dectl-review-demo
cd ~/dectl-review-demo
curl -fsSL https://raw.githubusercontent.com/jhonesis/dectl/main/scripts/dectl-demo.sh -o dectl-demo.sh
chmod +x dectl-demo.sh
dectl --version
```

Open `~/dectl-review-demo` as the workspace in your AI coding environment. Run the bootstrap and activate the isolated dectl home in its integrated terminal:

```bash
./dectl-demo.sh
source ./.dectl-demo-env.sh
```

The script runs `dectl project init --standard`, creates a React/Vite starter with Tailwind and shadcn/ui conventions, installs npm dependencies, and writes `.dectl-demo-guide.md`. The guide leads the agent through the rest of the walkthrough in the same workspace. `.dectl-demo-home/` keeps this demo's dectl database and trust settings separate from your normal home directory.

Ask your agent to read and follow `.dectl-demo-guide.md`. It will complete `specifications.md` from `brief.md`, ask questions, and show you the completed specification. **Review and approve that file before the agent runs `dectl spec init`.** Review and approve `specs/tasks.md` before implementation.

## What to Observe

- Does the agent ask for missing portfolio details instead of inventing personal or client information?
- Is the completed `specifications.md` presented for approval before spec generation?
- Are the generated specs and task breakdown clear enough to review and approve?
- Does `execute_task` keep implementation focused on the approved task and prepare understandable context?
- Do the configured `npm run build`, `npm test`, and `npm run lint` checks actually pass? Note the output; do not infer success from a summary alone.
- Does the final page show a responsive navbar, hero, portfolio/work section, About Us section, and footer? Is it readable and usable on mobile and desktop?
- Does the task state agree with the actual review result?

After a passing review, the guide starts the preview server. Open `http://localhost:5173` in the environment's browser or preview and inspect the rendered page.

## Feedback

Please include:

- AI environment/model and operating system
- Setup time and any installation or workspace friction
- Which step was clearest and which was confusing
- Whether the brief, specs, and approved task matched your expectations
- Build, test, lint, and reviewer results, including errors
- What you thought of the generated page at desktop and mobile widths
- Whether you would use this workflow on another project, and what would make it unsuitable

Do not add personal memory entries to your normal environment for this review; the demo's `.dectl-demo-home/` is isolated. Keep or remove the whole disposable folder when finished. Stop the local preview server before cleanup.

## Resources

- Landing page: https://dectl.4udev.download
- GitHub: https://github.com/jhonesis/dectl
- Documentation: https://deepwiki.com/jhonesis/dectl
- Contact: contact@4udev.download
