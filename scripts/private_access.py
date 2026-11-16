import json
import shutil

from invoke import Context

from phrase_gym.server.boundary import tailscale_host


def existing_tailscale_origin(context: Context) -> str:
    """Reuse only an existing private HTTPS route to this development server."""
    if not shutil.which("tailscale"):
        return ""
    result = context.run("tailscale serve status --json", hide=True, warn=True, pty=False)
    if result is None or not result.ok:
        return ""
    config = json.loads(result.stdout)
    if config.get("TCP", {}).get("443", {}).get("HTTPS") is not True:
        return ""
    for address, site in config.get("Web", {}).items():
        if not address.endswith(":443"):
            continue
        if site.get("Handlers", {}).get("/", {}).get("Proxy") != "http://127.0.0.1:8000":
            continue
        if config.get("AllowFunnel", {}).get(address):
            raise ValueError("Phrase Gym requires private Tailscale Serve; disable Funnel")
        origin = f"https://{address.removesuffix(':443')}"
        tailscale_host(origin)
        return origin
    return ""
