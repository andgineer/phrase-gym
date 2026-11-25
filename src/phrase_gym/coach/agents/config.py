_DEFAULT_AGENT_API_KEY_VARS: dict[str, list[str]] = {
    "claude": ["ANTHROPIC_API_KEY", "ANTHROPIC_AUTH_TOKEN"],
    "codex": ["OPENAI_API_KEY"],
    "gemini": ["ANTIGRAVITY_API_KEY", "GEMINI_API_KEY", "GOOGLE_API_KEY"],
}

# Pinned so card quality does not follow the model the learner last chose for everyday work.
CLAUDE_MODEL = "--model claude-opus-5-5 --effort medium"
CODEX_MODEL = "--model gpt-6.1-sol -c model_reasoning_effort=medium"
# Antigravity's free tier serves only Gemini Flash; the effort is part of the model name.
GEMINI_MODEL = "--model gemini-3.8-flash-low"
AGENT_LAUNCH_DELAY = {"gemini": 10.0}

_DEFAULT_CODEX_CMD = (
    "codex exec {model} --ephemeral --color never "
    "-c {default_permissions} -c {permission_profile} -c project_doc_max_bytes=0 "
    "--output-schema {schema_file} --output-last-message {answer_file} --cd {workspace} "
    '"Read your task from {prompt_file} and execute it."'
)
_DEFAULT_CLAUDE_CMD = (
    "claude -p {model} --output-format json --json-schema {schema} --no-session-persistence "
    "--tools Read --allowed-tools {read_scope} "
    '--disallowed-tools "Read(~/**)" "Read(//**/.env)" "Read(//**/.env.*)" '
    "--permission-mode manual --permission-prompts none --strict-mcp-config "
    "--setting-sources user "
    '-- "Read your task from {prompt_file} and execute it."'
)
# news-recap's Antigravity template; the runner adds --log-file. -p must come last.
_DEFAULT_GEMINI_CMD = (
    "agy {model} --dangerously-skip-permissions --output-format json "
    '-p "Read your task from {prompt_file} and execute it."'
)

# Coach starts from this allowlist instead of the whole parent environment.
INHERITED_ENVIRONMENT = (
    "HOME",
    "USER",
    "LOGNAME",
    "PATH",
    "SHELL",
    "TMPDIR",
    "LANG",
    "LC_ALL",
    "LC_CTYPE",
    "SSL_CERT_FILE",
    "SSL_CERT_DIR",
    "HTTP_PROXY",
    "HTTPS_PROXY",
    "NO_PROXY",
    "CODEX_HOME",
    "CLAUDE_CONFIG_DIR",
)
