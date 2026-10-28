import argparse
import os
from pathlib import Path

import uvicorn
from platformdirs import user_data_path

from phrase_gym.coach.main import run_coach
from phrase_gym.server.boundary import tailscale_host


def run(args: argparse.Namespace, parser: argparse.ArgumentParser) -> None:
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
    uvicorn.run("phrase_gym.server.app:create_app", factory=True, host=args.host, port=args.port)


def main() -> None:
    parser = argparse.ArgumentParser(description="Phrase Gym commands")
    commands = parser.add_subparsers(dest="command", required=True)
    run_parser = commands.add_parser("run", help="Run the local server")
    run_parser.add_argument("--host", default="127.0.0.1")
    run_parser.add_argument("--port", type=int, default=8000)
    run_parser.add_argument("--data-dir", type=Path)
    run_parser.add_argument("--tailscale-origin", type=str)
    coach_parser = commands.add_parser("coach", help="Run foreground Coach preparation")
    coach_parser.add_argument("--url", default="http://127.0.0.1:8000")
    coach_parser.add_argument("--once", action="store_true")
    for name in ("install-coach", "uninstall-coach", "deploy"):
        commands.add_parser(name, help="Planned command; not available yet")
    args = parser.parse_args()
    if args.command == "run":
        run(args, run_parser)
    elif args.command == "coach":
        run_coach(args.url, once=args.once)
    else:
        parser.error(f"{args.command} is not available yet")


if __name__ == "__main__":
    main()
