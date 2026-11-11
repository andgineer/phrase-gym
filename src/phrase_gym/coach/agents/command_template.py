import os
import shlex
import string
import subprocess
from pathlib import Path

from phrase_gym.coach.agents.subprocess import SubprocessError


def build_run_args(
    command_template: str,
    *,
    os_name: str | None = None,
    **values: str | Path,
) -> tuple[str | list[str], str]:
    stripped = command_template.strip()
    if not stripped:
        raise SubprocessError("Command template is empty.", transient=False)

    current_os_name = os_name or os.name
    str_values = {k: v.as_posix() if isinstance(v, Path) else str(v) for k, v in values.items()}
    try:
        if current_os_name == "nt":
            rendered = _render_windows_command_template(
                template=stripped,
                values=str_values,
            ).strip()
            if not rendered:
                raise SubprocessError(
                    "Command template rendered empty command.",
                    transient=False,
                )
            command_head = rendered.split(maxsplit=1)[0]
            return rendered, command_head

        unix_values = {
            k: shlex.quote(v.as_posix()) if isinstance(v, Path) else str(v)
            for k, v in values.items()
        }
        rendered = stripped.format(**unix_values)
    except KeyError as error:
        raise SubprocessError(
            f"Unsupported command template placeholder: {error}",
            transient=False,
        ) from error

    argv = shlex.split(rendered)
    if not argv:
        raise SubprocessError("Command template rendered empty command.", transient=False)
    return argv, argv[0]


def _render_windows_command_template(*, template: str, values: dict[str, str]) -> str:
    formatter = string.Formatter()
    rendered_parts: list[str] = []
    in_double_quotes = False

    for literal_text, field_name, format_spec, conversion in formatter.parse(template):
        rendered_parts.append(literal_text)
        in_double_quotes = _advance_windows_quote_state(literal_text, in_double_quotes)
        if field_name is None:
            continue

        try:
            value = values[field_name]
        except KeyError as error:
            raise KeyError(field_name) from error

        value_text = _apply_string_conversion(value, conversion, format_spec)
        if in_double_quotes:
            rendered_parts.append(_escape_windows_embedded_quote_value(value_text))
            continue

        rendered_parts.append(subprocess.list2cmdline([value_text]))

    return "".join(rendered_parts)


def _apply_string_conversion(
    value: str,
    conversion: str | None,
    format_spec: str | None,
) -> str:
    converted: str
    if conversion == "r":
        converted = repr(value)
    elif conversion == "a":
        converted = ascii(value)
    elif conversion in (None, "", "s"):
        converted = str(value)
    else:
        raise ValueError(f"Unsupported format conversion: !{conversion}")

    if format_spec:
        return format(converted, format_spec)
    return converted


def _advance_windows_quote_state(literal_text: str, in_double_quotes: bool) -> bool:
    for index, char in enumerate(literal_text):
        if char != '"':
            continue
        backslashes = 0
        scan_index = index - 1
        while scan_index >= 0 and literal_text[scan_index] == "\\":
            backslashes += 1
            scan_index -= 1
        if backslashes % 2 == 1:
            continue
        in_double_quotes = not in_double_quotes
    return in_double_quotes


def _escape_windows_embedded_quote_value(value: str) -> str:
    return value.replace('"', '\\"')
