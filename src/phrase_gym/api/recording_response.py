from fastapi import Response


def recording_response(
    audio: bytes,
    mime_type: str,
    sha256: str,
    requested: str | None,
) -> Response:
    headers = {
        "ETag": f'"{sha256}"',
        "Cache-Control": "private, max-age=31536000, immutable",
        "Accept-Ranges": "bytes",
    }
    if requested is None:
        return Response(audio, media_type=mime_type, headers=headers)
    try:
        unit, bounds = requested.split("=", 1)
        start_text, end_text = bounds.split("-", 1)
        if unit != "bytes" or "," in bounds:
            raise ValueError
        start = int(start_text) if start_text else max(0, len(audio) - int(end_text))
        end = min(int(end_text), len(audio) - 1) if end_text and start_text else len(audio) - 1
        if start < 0 or start > end or start >= len(audio):
            raise ValueError
    except ValueError:
        return Response(
            status_code=416,
            headers={**headers, "Content-Range": f"bytes */{len(audio)}"},
        )
    headers["Content-Range"] = f"bytes {start}-{end}/{len(audio)}"
    return Response(audio[start : end + 1], status_code=206, media_type=mime_type, headers=headers)
