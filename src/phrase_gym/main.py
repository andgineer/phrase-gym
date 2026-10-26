import argparse
import os
from pathlib import Path

import uvicorn
from platformdirs import user_data_path

from phrase_gym.boundary import tailscale_host


def main() -> None:
    parser = argparse.ArgumentParser(description="Run Phrase Gym on this computer")
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8000)
    parser.add_argument("--data-dir", type=Path)
    parser.add_argument("--tailscale-origin", type=str)
    args = parser.parse_args()
    if args.host not in {"127.0.0.1", "localhost"}:
        parser.error("Phrase Gym only listens on loopback")
    if args.tailscale_origin:
        try:
            tailscale_host(args.tailscale_origin)
        except ValueError as error:
            parser.error(str(error))
        os.environ["PHRASE_GYM_TAILSCALE_ORIGIN"] = args.tailscale_origin
    if args.data_dir:
        os.environ["PHRASE_GYM_DATA_DIR"] = str(args.data_dir.resolve())
    else:
        os.environ.setdefault("PHRASE_GYM_DATA_DIR", str(user_data_path("phrase-gym")))
    uvicorn.run("phrase_gym.app:create_app", factory=True, host=args.host, port=args.port)


if __name__ == "__main__":
    main()
