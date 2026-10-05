#!/usr/bin/env bash
# Download this script into a new, disposable folder used only for the dectl
# demo, then open that folder in your AI coding environment. The demo creates
# its project files beside this script; do not run it from a real project.
set -euo pipefail

DECTL_BIN="$(command -v dectl || true)"
if [[ -z "$DECTL_BIN" ]]; then
  echo "dectl is not installed or is not on PATH. Install it, then run this demo again."
  exit 1
fi

DEMO_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
DEMO_HOME="$DEMO_ROOT/.dectl-demo-home"
DEMO_PROJECT="$DEMO_ROOT"

for generated_path in \
   .dec \
   specifications.md \
   brief.md \
   package.json \
   package-lock.json \
   node_modules \
   index.html \
   vite.config.js \
   tailwind.config.js \
   postcss.config.js \
   components.json \
   src/App.jsx \
   src/components/ui/button.jsx \
   tests/App.test.jsx \
   .dectl-demo-home \
   .dectl-demo-env.sh \
   .dectl-demo-guide.md \
   project-init.log; do
   if [[ -e "$DEMO_ROOT/$generated_path" ]]; then
      echo "Demo files already exist at $DEMO_ROOT/$generated_path."
      echo "Use a new, empty folder for each demo run; no existing files were changed."
      exit 1
   fi
done

if ! command -v node >/dev/null 2>&1 || ! command -v npm >/dev/null 2>&1; then
   echo "The landing-page demo requires Node.js 18+ and npm."
   exit 1
fi
NODE_MAJOR="$(node -p 'Number(process.versions.node.split(".")[0])')"
if [[ "$NODE_MAJOR" -lt 18 ]]; then
   echo "The landing-page demo requires Node.js 18 or newer (found $(node --version))."
   exit 1
fi

mkdir -p "$DEMO_HOME" "$DEMO_PROJECT/src/components/ui" "$DEMO_PROJECT/src/lib" "$DEMO_PROJECT/tests"

# Keep dectl's database, config, and trust registry inside the disposable sandbox.
export HOME="$DEMO_HOME"
cd "$DEMO_PROJECT"

cat > brief.md <<'EOF'
# Portfolio Landing Page

Build a small, polished, mobile-first portfolio landing page for a creative
professional or studio. It should include a responsive navbar, a hero section,
a selected portfolio/work section, an About Us section, and a footer with contact
and social links. Keep the page focused and easy to scan on a phone and desktop.

Use React, Tailwind CSS, and shadcn/ui components. Choose a professional visual
direction with an intentional color palette and distinctive typography; avoid a
generic template. Keep interactions functional and accessible, including mobile
navigation and clear calls to action. Do not add a backend or non-functional form.

Before writing specs, ask the developer for any missing portfolio identity,
headline, biography, project descriptions, contact details, color preferences,
and image assets. Do not invent personal details or claim work that was not
provided. If imagery is unavailable, ask whether remote sample images are okay
or use a deliberate typographic layout that does not imply real client work.
EOF

cat > package.json <<'EOF'
{
   "name": "portfolio-demo",
   "private": true,
   "version": "0.1.0",
   "type": "module",
   "scripts": {
      "dev": "vite --host 0.0.0.0",
      "build": "vite build",
      "test": "vitest run",
      "lint": "eslint src --ext .js,.jsx"
   },
   "dependencies": {
      "@radix-ui/react-slot": "^1.1.2",
      "class-variance-authority": "^0.7.1",
      "clsx": "^2.1.1",
      "lucide-react": "^0.468.0",
      "react": "^18.3.1",
      "react-dom": "^18.3.1",
      "tailwind-merge": "^2.6.0"
   },
   "devDependencies": {
      "@testing-library/jest-dom": "^6.6.3",
      "@testing-library/react": "^16.1.0",
      "@vitejs/plugin-react": "^4.3.4",
      "autoprefixer": "^10.4.20",
      "eslint": "^8.57.1",
      "eslint-plugin-react": "^7.37.2",
      "eslint-plugin-react-hooks": "^5.1.0",
      "eslint-plugin-react-refresh": "^0.4.16",
      "jsdom": "^25.0.1",
      "postcss": "^8.5.1",
      "tailwindcss": "^3.4.17",
      "vite": "^6.0.7",
      "vitest": "^2.1.8"
   }
}
EOF

cat > index.html <<'EOF'
<!doctype html>
<html lang="en">
   <head>
      <meta charset="UTF-8" />
      <meta name="viewport" content="width=device-width, initial-scale=1.0" />
      <meta name="theme-color" content="#f4f1eb" />
      <title>Portfolio | Demo</title>
   </head>
   <body>
      <div id="root"></div>
      <script type="module" src="/src/main.jsx"></script>
   </body>
</html>
EOF

cat > vite.config.js <<'EOF'
import path from "node:path";
import { fileURLToPath } from "node:url";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vitest/config";

const rootDir = path.dirname(fileURLToPath(import.meta.url));

export default defineConfig({
   plugins: [react()],
   resolve: {
      alias: { "@": path.resolve(rootDir, "src") },
   },
   test: {
      environment: "jsdom",
      setupFiles: "./src/test-setup.js",
   },
});
EOF

cat > tailwind.config.js <<'EOF'
/** @type {import('tailwindcss').Config} */
export default {
   darkMode: ["class"],
   content: ["./index.html", "./src/**/*.{js,jsx}"],
   theme: {
      extend: {
         colors: {
            border: "hsl(var(--border))",
            background: "hsl(var(--background))",
            foreground: "hsl(var(--foreground))",
            primary: "hsl(var(--primary))",
            "primary-foreground": "hsl(var(--primary-foreground))",
            muted: "hsl(var(--muted))",
            "muted-foreground": "hsl(var(--muted-foreground))",
         },
         borderRadius: { lg: "var(--radius)" },
      },
   },
   plugins: [],
};
EOF

cat > postcss.config.js <<'EOF'
export default {
   plugins: {
      tailwindcss: {},
      autoprefixer: {},
   },
};
EOF

cat > components.json <<'EOF'
{
   "$schema": "https://ui.shadcn.com/schema.json",
   "style": "new-york",
   "rsc": false,
   "tsx": false,
   "tailwind": {
      "config": "tailwind.config.js",
      "css": "src/index.css",
      "baseColor": "stone",
      "cssVariables": true,
      "prefix": ""
   },
   "aliases": {
      "components": "@/components",
      "utils": "@/lib/utils",
      "ui": "@/components/ui",
      "lib": "@/lib",
      "hooks": "@/hooks"
   }
}
EOF

cat > .eslintrc.json <<'EOF'
{
   "env": { "browser": true, "es2021": true },
   "extends": ["eslint:recommended", "plugin:react/recommended", "plugin:react-hooks/recommended"],
   "parserOptions": { "ecmaVersion": "latest", "sourceType": "module", "ecmaFeatures": { "jsx": true } },
   "plugins": ["react-refresh"],
   "settings": { "react": { "version": "detect" } },
   "rules": {
      "react/prop-types": "off",
      "react/react-in-jsx-scope": "off",
      "react-refresh/only-export-components": "off"
   }
}
EOF

cat > src/index.css <<'EOF'
@tailwind base;
@tailwind components;
@tailwind utilities;

:root {
   font-family: "DM Sans", sans-serif;
   color-scheme: light;
   --background: 40 20% 96%;
   --foreground: 24 10% 12%;
   --primary: 161 45% 28%;
   --primary-foreground: 40 20% 96%;
   --muted: 36 14% 90%;
   --muted-foreground: 25 8% 38%;
   --border: 34 12% 82%;
   --radius: 0.5rem;
   background: hsl(var(--background));
   color: hsl(var(--foreground));
   font-synthesis: none;
   text-rendering: optimizeLegibility;
}

* {
   box-sizing: border-box;
}

body {
   min-width: 320px;
   min-height: 100vh;
   margin: 0;
}

button,
a {
   -webkit-tap-highlight-color: transparent;
}
EOF

cat > src/lib/utils.js <<'EOF'
import { clsx } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs) {
   return twMerge(clsx(inputs));
}
EOF

cat > src/components/ui/button.jsx <<'EOF'
import { Slot } from "@radix-ui/react-slot";
import { cva } from "class-variance-authority";
import { cn } from "@/lib/utils";

const buttonVariants = cva(
   "inline-flex items-center justify-center gap-2 whitespace-nowrap rounded-lg text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-primary disabled:pointer-events-none disabled:opacity-50",
   {
      variants: {
         variant: {
            default: "bg-primary text-primary-foreground hover:bg-primary/90",
            outline: "border border-border bg-transparent hover:bg-muted",
            ghost: "hover:bg-muted",
         },
         size: {
            default: "min-h-11 px-5 py-2",
            sm: "min-h-9 rounded-md px-3",
            icon: "size-11",
         },
      },
      defaultVariants: { variant: "default", size: "default" },
   },
);

export function Button({ className, variant, size, asChild = false, ...props }) {
   const Component = asChild ? Slot : "button";
   return (
      <Component
         className={cn(buttonVariants({ variant, size }), className)}
         {...props}
      />
   );
}
EOF

cat > src/App.jsx <<'EOF'
import { Button } from "@/components/ui/button";

export default function App() {
   return (
      <div className="min-h-screen bg-background text-foreground">
         <header className="border-b border-border">
            <nav aria-label="Primary navigation" className="mx-auto flex max-w-6xl items-center justify-between px-5 py-4">
               <a className="font-semibold" href="#top">Portfolio</a>
               <div className="flex items-center gap-5 text-sm">
                  <a href="#work">Work</a>
                  <a href="#about">About us</a>
               </div>
            </nav>
         </header>
         <main id="top">
            <section aria-labelledby="hero-title" className="mx-auto grid min-h-[65vh] max-w-6xl content-center gap-6 px-5 py-16">
               <p className="text-sm font-medium uppercase tracking-[0.16em] text-primary">Independent creative portfolio</p>
               <h1 id="hero-title" className="max-w-4xl text-5xl font-semibold leading-[1.05] sm:text-7xl">
                  A considered portfolio starts here.
               </h1>
               <p className="max-w-xl text-lg text-muted-foreground">A starter canvas for thoughtful work and a clear point of view.</p>
               <div><Button asChild><a href="#work">Explore selected work</a></Button></div>
            </section>
            <section id="work" aria-label="Selected work" className="mx-auto max-w-6xl px-5 py-16">
               <h2 className="text-3xl font-semibold">Selected work</h2>
               <p className="mt-3 text-muted-foreground">Project stories will be shaped with the developer.</p>
            </section>
            <section id="about" aria-label="About us" className="mx-auto max-w-6xl border-t border-border px-5 py-16">
               <h2 className="text-3xl font-semibold">About us</h2>
               <p className="mt-3 max-w-2xl text-muted-foreground">Add an approved introduction to the person or studio behind this portfolio.</p>
            </section>
         </main>
         <footer className="border-t border-border px-5 py-8 text-sm text-muted-foreground">
            <div className="mx-auto flex max-w-6xl flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
               <span>Portfolio demo</span>
               <a href="mailto:hello@example.com">Get in touch</a>
            </div>
         </footer>
      </div>
   );
}
EOF

cat > src/main.jsx <<'EOF'
import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App.jsx";
import "./index.css";

ReactDOM.createRoot(document.getElementById("root")).render(
   <React.StrictMode>
      <App />
   </React.StrictMode>,
);
EOF

cat > src/test-setup.js <<'EOF'
import "@testing-library/jest-dom/vitest";
EOF

cat > tests/App.test.jsx <<'EOF'
import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import App from "../src/App.jsx";

describe("portfolio landing page structure", () => {
   it("provides navigation, hero, selected work, about, and footer", () => {
      render(<App />);

      expect(screen.getByRole("navigation", { name: "Primary navigation" })).toBeInTheDocument();
      expect(screen.getByRole("heading", { level: 1 })).toBeInTheDocument();
      expect(screen.getByRole("region", { name: "Selected work" })).toBeInTheDocument();
      expect(screen.getByRole("region", { name: "About us" })).toBeInTheDocument();
      expect(screen.getByRole("contentinfo")).toBeInTheDocument();
   });
});
EOF

echo "Installing the isolated React, Tailwind, shadcn/ui, test, and lint dependencies..."
npm install --no-audit --no-fund

echo "dectl $("$DECTL_BIN" --version)"
echo "Demo folder: $DEMO_PROJECT"
echo "Isolated dectl home: $DEMO_HOME"
echo
echo "Developer: provide the project brief; the demo created brief.md."
echo "dectl: create the starter context and specifications template."
INIT_LOG="$DEMO_ROOT/project-init.log"
"$DECTL_BIN" project init --standard > "$INIT_LOG"
echo "✓ dectl project init --standard completed. Initialization details: $INIT_LOG"
CONFIG_TMP="$(mktemp "$DEMO_ROOT/project.toml.XXXXXX")"
sed \
   -e '/^\[build\]/,/^\[/{s/^command = ""$/command = "npm run build"/;}' \
   -e '/^\[test\]/,/^\[/{s/^command = ""$/command = "npm test"/;}' \
   -e '/^\[lint\]/,/^\[/{s/^command = ""$/command = "npm run lint"/;}' \
  .dec/config/project.toml > "$CONFIG_TMP"
mv "$CONFIG_TMP" .dec/config/project.toml
grep -q '^command = "npm run build"$' .dec/config/project.toml
grep -q '^command = "npm test"$' .dec/config/project.toml
grep -q '^command = "npm run lint"$' .dec/config/project.toml
"$DECTL_BIN" project info

echo
printf 'export HOME=%q\ncd %q\n' "$DEMO_HOME" "$DEMO_PROJECT" > .dectl-demo-env.sh

cat > .dectl-demo-guide.md <<'EOF'
# dectl Agent Walkthrough

Run this walkthrough in the current AI-agent session and terminal. Do not open a
second IDE window. Keep the current `HOME` pointed at this demo's temporary home.

## Narration and roles

Before each stage, briefly tell the developer what is about to happen and name
the responsible actor:
- **Developer** provides the brief, answers clarifying questions, reviews the
  generated specs, and approves the task to demonstrate.
- **AI agent** completes the specification template, creates the specs, implements
  the approved task, and explains the observed results.
- **dectl** prepares project context, supplies task-scoped context and rules,
  coordinates the workflow, and runs configured checks.

Do not skip the developer's clarification or approval points. Do not claim a
check passed unless its command actually passed. If review fails, show the failure
and stop the walkthrough without presenting the task as successful, even if a
later documenter step records the attempt.

## 1. Turn the brief into specs

1. Read `brief.md` and `specifications.md`.
2. Complete `specifications.md` from `brief.md`. Ask the developer any questions
   needed for the portfolio identity, role, biography, project details, contact
   links, visual direction, and image assets. Wait for answers before proceeding.
3. Show the completed `specifications.md` to the developer and ask whether they
   approve its contents. Wait for explicit approval; if they request changes,
   revise the file and show it again. Do not create specs before approval.
4. After approval, explain that dectl will prepare the SDD input, then run:

   ```bash
   dectl spec init --from specifications.md
   ```

5. Follow `.dec/sdd/SKILL.md`, `.dec/sdd/references/templates.md`, and
   `.dec/sdd/references/examples.md` to create the specs. The developer owns the
   design decisions; ask them to review and approve `specs/tasks.md` before code.

## 2. Run one approved task

1. Find the first pending task that will make the portfolio landing page visibly
   functional. If the plan has prerequisite tasks, complete them in order until
   that task is ready. Show its ID, description, Build, Verify, and Gate to the
   developer; ask them to approve it. Use its actual ID and description below.
2. Start the task workflow, narrating that dectl is collecting project, memory,
   decision, and task-rule context:

   ```bash
   dectl workflow run execute_task --var task_id=<TASK_ID> --var description="<TASK_DESCRIPTION>" --auto
   ```

3. The workflow pauses for implementation. Explain that the AI agent now owns
   the code change, then implement only the approved task using the prepared
   context in `.dec/agent-output/`. Follow the approved visual direction; keep
   the page mobile-first, accessible, and based on the configured Tailwind and
   shadcn/ui setup. Do not invent personal details or change unrelated files.
4. Run the resume command printed by the workflow, normally:

   ```bash
   dectl workflow run execute_task --var task_id=<TASK_ID> --var description="<TASK_DESCRIPTION>" --from-step 4 --auto
   ```

5. Show the actual build, test, review, and rules results. Report failures
   accurately; do not infer success from the existence of a report or memory entry.

## 3. Preview the landing page

After the review passes, start the development server:

```bash
npm run dev -- --host 0.0.0.0
```

Keep the server running and show the developer `http://localhost:5173` in the
environment's browser or preview.

## 4. Inspect the outcome

Show the review report in `.dec/agent-output/`, the matching task entry in
`progress.json` and `specs/tasks.md`, and the task's memory search result. Explain
which checks ran and which actor performed each step. Keep the sandbox and its
isolated memory for inspection; the developer can remove the printed sandbox path
when finished.
EOF

echo
echo "Demo ready in the folder containing this script: $DEMO_PROJECT"
echo "Isolated dectl home: $DEMO_HOME"
echo "The setup is complete; this script will now exit."
echo "In the AI-agent terminal opened at this folder, activate the isolated home and read the guide:"
printf '  source %q\n' "$DEMO_PROJECT/.dectl-demo-env.sh"
echo "  cat .dectl-demo-guide.md"
echo
echo "All demo project files will be created here; no second demo window is needed."
