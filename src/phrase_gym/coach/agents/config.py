_DEFAULT_AGENT_API_KEY_VARS: dict[str, list[str]] = {
    "claude": ["ANTHROPIC_API_KEY", "ANTHROPIC_AUTH_TOKEN"],
    "codex": ["OPENAI_API_KEY"],
}

# {model} stays empty: each CLI uses the learner's own default model and settings.
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
    "--permission-prompts none --strict-mcp-config --setting-sources user "
    '-- "Read your task from {prompt_file} and execute it."'
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
)
