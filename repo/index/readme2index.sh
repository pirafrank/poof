#!/usr/bin/env bash

# readme2index.sh
# Converts a Markdown README to index.html using the same HTML/CSS
# design as generate-pkg-index.sh
#
# Usage: ./readme2index.sh > index.html
# Or:    ./readme2index.sh --input path/to/README.md --app-name poof > index.html
# Or:    README=/path/to/README.md APP_NAME=poof ./readme2index.sh > index.html

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# ---------------------------------------------------------------------------
# Defaults (can be overridden via env vars or CLI flags)
# ---------------------------------------------------------------------------

README_FILE="${README:-${SCRIPT_DIR}/../README.md}"
APP_NAME="${APP_NAME:-poof}"
MAINTAINER="${MAINTAINER:-Francesco Pira <dev@fpira.com>}"
MAINTAINER_URL="${MAINTAINER_URL:-https://github.com/pirafrank/${APP_NAME}}"

# Parse optional CLI flags
while [[ $# -gt 0 ]]; do
  case "$1" in
    --input|-i)       README_FILE="$2";    shift 2 ;;
    --app-name)       APP_NAME="$2";       shift 2 ;;
    --maintainer)     MAINTAINER="$2";     shift 2 ;;
    --maintainer-url) MAINTAINER_URL="$2"; shift 2 ;;
    --help|-h)
      echo "Usage: $0 [--input README.md] [--app-name NAME] [--maintainer NAME] [--maintainer-url URL]"
      exit 0 ;;
    *) echo "Unknown argument: $1" >&2; exit 1 ;;
  esac
done

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------

html_escape() {
  # Use sed so that '&' in replacements is treated as a literal character
  # (bash ${var//pat/rep} treats '&' as the matched string, like sed does
  # without escaping — so we avoid bash substitution here entirely).
  printf '%s' "$1" \
    | sed \
        -e 's/&/\&amp;/g' \
        -e 's/</\&lt;/g' \
        -e 's/>/\&gt;/g' \
        -e 's/"/\&quot;/g'
}

# Process inline Markdown: bold, inline code, links.
# Chains all sed transforms in one subshell to keep it fast.
inline_md() {
  local s
  s="$(html_escape "$1")"
  printf '%s' "$s" \
    | sed -E \
        -e 's/\*\*([^*]+)\*\*/<strong>\1<\/strong>/g' \
        -e "s/\`([^\`]+)\`/<code class=\"inline\">\1<\/code>/g" \
        -e 's/\[([^]]+)\]\(([^)]+)\)/<a href="\2">\1<\/a>/g'
}

slugify() {
  printf '%s' "$1" \
    | tr '[:upper:]' '[:lower:]' \
    | sed 's/[^a-z0-9]/-/g; s/--*/-/g; s/^-//; s/-$//'
}

CODE_ID=0
emit_code_block() {
  local raw="$1"
  CODE_ID=$((CODE_ID + 1))
  local cid="codeblock-${CODE_ID}"
  local content
  content="$(html_escape "${raw%$'\n'}")"
  cat <<BLOCK
        <div class="code-block standalone">
          <button class="copy-btn" data-target="${cid}" aria-label="Copy to clipboard">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="none"
                 stroke="currentColor" stroke-width="2" stroke-linecap="round"
                 stroke-linejoin="round" width="16" height="16">
              <rect x="9" y="9" width="13" height="13" rx="2" ry="2"/>
              <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
            </svg>
            Copy
          </button>
          <pre><code id="${cid}">${content}</code></pre>
        </div>
BLOCK
}

# ---------------------------------------------------------------------------
# Pass 1: extract page title (h1) and intro paragraphs (before first h2)
# ---------------------------------------------------------------------------

PAGE_TITLE=""
INTRO_HTML=""

extract_front_matter() {
  local state="pre_h1"
  local in_para=false

  while IFS= read -r line; do
    if [[ "$state" == "pre_h1" ]]; then
      if [[ "$line" =~ ^#[[:space:]](.+)$ ]]; then
        PAGE_TITLE="${BASH_REMATCH[1]}"
        state="intro"
      fi
      continue
    fi

    # Stop collecting intro at first h2
    if [[ "$line" =~ ^##[[:space:]] ]]; then
      break
    fi

    if [[ -z "$line" ]]; then
      if $in_para; then
        INTRO_HTML+="</p>"
        in_para=false
      fi
    else
      if ! $in_para; then
        INTRO_HTML+="<p>"
        in_para=true
      else
        INTRO_HTML+=" "
      fi
      INTRO_HTML+="$(inline_md "$line")"
    fi
  done < "$README_FILE"

  if $in_para; then
    INTRO_HTML+="</p>"
  fi
}

# ---------------------------------------------------------------------------
# Emit HTML head + header
# ---------------------------------------------------------------------------

emit_head() {
  local title_esc
  title_esc="$(html_escape "$PAGE_TITLE")"
  cat <<HTML_HEAD
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>${APP_NAME} packages</title>
  <style>
    *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }

    :root {
      --bg:          #0f1117;
      --surface:     #1a1d27;
      --surface2:    #22263a;
      --border:      #2e3350;
      --accent:      #f6821f;
      --accent-dim:  #c8650f;
      --text:        #e2e4ef;
      --text-muted:  #8b8fa8;
      --code-bg:     #12141e;
      --code-text:   #c9d1d9;
      --tab-active:  #f6821f;
      --radius:      8px;
      --font-mono:   "JetBrains Mono", "Fira Code", "Cascadia Code", monospace;
      --font-sans:   -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
    }

    body {
      background: var(--bg);
      color: var(--text);
      font-family: var(--font-sans);
      font-size: 15px;
      line-height: 1.6;
      padding: 0 16px 64px;
    }

    header {
      max-width: 860px;
      margin: 0 auto;
      padding: 40px 0 24px;
      border-bottom: 1px solid var(--border);
    }

    header h1 {
      font-size: 1.9rem;
      font-weight: 700;
      color: var(--text);
    }

    header h1 span { color: var(--accent); }

    header p {
      margin-top: 8px;
      color: var(--text-muted);
      font-size: 0.95rem;
    }

    header a { color: var(--accent); text-decoration: none; }
    header a:hover { text-decoration: underline; }

    .container {
      max-width: 860px;
      margin: 0 auto;
    }

    .pkg-section {
      margin-top: 48px;
    }

    .pkg-section h2 {
      font-size: 1.15rem;
      font-weight: 600;
      letter-spacing: 0.03em;
      color: var(--text-muted);
      text-transform: uppercase;
      margin-bottom: 16px;
    }

    .pkg-section h3 {
      font-size: 1rem;
      font-weight: 600;
      color: var(--text);
      margin: 24px 0 10px;
    }

    .pkg-section p {
      color: var(--text);
      margin-bottom: 12px;
    }

    .pkg-section ul {
      list-style: disc;
      margin: 4px 0 16px 24px;
      color: var(--text);
    }

    .pkg-section li { margin-bottom: 4px; }

    .pkg-section a { color: var(--accent); text-decoration: none; }
    .pkg-section a:hover { text-decoration: underline; }

    code.inline {
      font-family: var(--font-mono);
      font-size: 0.85em;
      background: var(--surface2);
      border: 1px solid var(--border);
      border-radius: 3px;
      padding: 1px 5px;
      color: var(--code-text);
    }

    /* Tab bar */
    .tabs {
      display: flex;
      flex-wrap: wrap;
      gap: 4px;
      border-bottom: 2px solid var(--border);
      margin-bottom: 0;
    }

    .tabs button {
      background: none;
      border: none;
      border-bottom: 2px solid transparent;
      margin-bottom: -2px;
      padding: 8px 16px;
      color: var(--text-muted);
      font-size: 0.88rem;
      font-family: var(--font-sans);
      cursor: pointer;
      border-radius: var(--radius) var(--radius) 0 0;
      transition: color 0.15s, border-color 0.15s, background 0.15s;
    }

    .tabs button:hover {
      color: var(--text);
      background: var(--surface2);
    }

    .tabs button.active {
      color: var(--tab-active);
      border-bottom-color: var(--tab-active);
      font-weight: 600;
    }

    .tab-panel { display: block; }
    .tab-panel[hidden] { display: none; }

    /* Code block (used by tab panels in generate-pkg-index output) */
    .code-block {
      position: relative;
      background: var(--code-bg);
      border: 1px solid var(--border);
      border-top: none;
      border-radius: 0 0 var(--radius) var(--radius);
      padding: 20px;
    }

    /* Standalone code block — not beneath a tab bar */
    .code-block.standalone {
      border-top: 1px solid var(--border);
      border-radius: var(--radius);
      margin: 4px 0 20px;
    }

    .code-block pre { overflow-x: auto; }

    .code-block code {
      font-family: var(--font-mono);
      font-size: 0.82rem;
      line-height: 1.7;
      color: var(--code-text);
      white-space: pre;
    }

    /* Copy button */
    .copy-btn {
      position: absolute;
      top: 12px;
      right: 12px;
      display: flex;
      align-items: center;
      gap: 5px;
      padding: 5px 10px;
      background: var(--surface2);
      border: 1px solid var(--border);
      border-radius: 5px;
      color: var(--text-muted);
      font-size: 0.78rem;
      font-family: var(--font-sans);
      cursor: pointer;
      transition: background 0.15s, color 0.15s;
    }

    .copy-btn:hover { background: var(--border); color: var(--text); }
    .copy-btn.copied { color: #4caf50; border-color: #4caf50; }

    @media (max-width: 600px) {
      .tabs button { padding: 6px 10px; font-size: 0.8rem; }
    }
  </style>
</head>
<body>
  <header>
    <h1><span>${APP_NAME}</span> ${title_esc}</h1>
    ${INTRO_HTML}
  </header>

  <main class="container">
HTML_HEAD
}

# ---------------------------------------------------------------------------
# Pass 2: emit body sections (h2 → .pkg-section, h3, lists, code blocks)
# ---------------------------------------------------------------------------

emit_body() {
  local in_code=false
  local code_buf=""
  local in_list=false
  local in_para=false
  local in_section=false
  local past_h1=false
  local past_first_h2=false
  local h2_text sid h3_text

  close_inline() {
    if $in_list; then echo "        </ul>"; in_list=false; fi
    if $in_para; then echo "        </p>"; in_para=false; fi
  }

  while IFS= read -r line; do
    # Skip everything up to and including the h1 title line
    if ! $past_h1; then
      if [[ "$line" =~ ^#[[:space:]] ]]; then
        past_h1=true
      fi
      continue
    fi

    # Skip intro paragraphs (content before the first h2)
    if ! $past_first_h2; then
      if [[ "$line" =~ ^##[[:space:]] ]]; then
        past_first_h2=true
        # Fall through to h2 handling below
      else
        continue
      fi
    fi

    # ---- Fenced code block ----
    if $in_code; then
      if [[ "$line" == '```'* ]]; then
        in_code=false
        emit_code_block "$code_buf"
        code_buf=""
      else
        code_buf+="${line}"$'\n'
      fi
      continue
    fi

    if [[ "$line" == '```'* ]]; then
      close_inline
      in_code=true
      code_buf=""
      continue
    fi

    # ---- h3 (checked before h2 — more specific pattern) ----
    if [[ "$line" =~ ^###[[:space:]](.+)$ ]]; then
      h3_text="${BASH_REMATCH[1]}"
      close_inline
      echo "        <h3>$(inline_md "$h3_text")</h3>"
      continue
    fi

    # ---- h2: open a new section ----
    if [[ "$line" =~ ^##[[:space:]](.+)$ ]]; then
      h2_text="${BASH_REMATCH[1]}"
      sid="$(slugify "$h2_text")"
      close_inline
      if $in_section; then echo "      </section>"; fi
      in_section=true
      echo "      <section class=\"pkg-section\" id=\"${sid}\">"
      echo "        <h2>$(inline_md "$h2_text")</h2>"
      continue
    fi

    # ---- Blank line: close open inline blocks ----
    if [[ -z "$line" ]]; then
      close_inline
      continue
    fi

    # ---- List item ----
    if [[ "$line" =~ ^-[[:space:]](.+)$ ]]; then
      if $in_para; then echo "        </p>"; in_para=false; fi
      if ! $in_list; then echo "        <ul>"; in_list=true; fi
      echo "          <li>$(inline_md "${BASH_REMATCH[1]}")</li>"
      continue
    fi

    # ---- Paragraph text ----
    if $in_list; then echo "        </ul>"; in_list=false; fi
    if ! $in_para; then echo "        <p>"; in_para=true; fi
    echo "          $(inline_md "$line")"

  done < "$README_FILE"

  # Flush any open inline blocks at end of file
  close_inline
  if $in_section; then echo "      </section>"; fi
}

# ---------------------------------------------------------------------------
# Emit HTML foot
# ---------------------------------------------------------------------------

emit_foot() {
  cat <<HTML_FOOT

  </main>

  <script>
    // Tab switching (shared with generate-pkg-index.sh output)
    document.querySelectorAll('.tabs').forEach(function(tabBar) {
      tabBar.addEventListener('click', function(e) {
        const btn = e.target.closest('button[role="tab"]');
        if (!btn) return;
        const section = tabBar.closest('.pkg-section');
        tabBar.querySelectorAll('button').forEach(function(b) {
          b.classList.remove('active');
          b.setAttribute('aria-selected', 'false');
        });
        section.querySelectorAll('.tab-panel').forEach(function(p) {
          p.hidden = true;
        });
        btn.classList.add('active');
        btn.setAttribute('aria-selected', 'true');
        const target = section.querySelector('#' + btn.dataset.target);
        if (target) target.hidden = false;
      });
    });

    // Copy to clipboard
    document.querySelectorAll('.copy-btn').forEach(function(btn) {
      btn.addEventListener('click', function() {
        const codeEl = document.getElementById(btn.dataset.target);
        if (!codeEl) return;
        navigator.clipboard.writeText(codeEl.textContent).then(function() {
          btn.classList.add('copied');
          const original = btn.innerHTML;
          btn.innerHTML = btn.innerHTML.replace('Copy', 'Copied!');
          setTimeout(function() {
            btn.classList.remove('copied');
            btn.innerHTML = original;
          }, 2000);
        });
      });
    });
  </script>
</body>
</html>
HTML_FOOT
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

extract_front_matter
emit_head
emit_body
emit_foot
