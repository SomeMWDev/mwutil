import os
from pathlib import Path

import argcomplete
import argparse
import importlib

from mwutil.module import MWUtilModule
from mwutil.utils import find_mwutil_config, load_mwutil_config, load_core_env


def main():
    parser = argparse.ArgumentParser(description="Manage MediaWiki development environments")

    subparsers = parser.add_subparsers(help="Run a module")

    module_names = [
        "add-gerrit-ssh-key",
        "bash",
        "down",
        "dump",
        "elasticsearch",
        "lint",
        "list-repo-remotes",
        "phan",
        "phpunit",
        "pull",
        "recreate",
        "reset",
        "run",
        "security",
        "setup-gerrit",
        "setup-github",
        "shell",
        "sql",
        "up",
        "update"
    ]

    debug = os.getenv("MWUTIL_DEBUG")

    # Detect if we're in a mediawiki installation
    try:
        basedir: Path = find_mwutil_config()
        if debug:
            print(f"Found .mwutil.json in {basedir}")
    except FileNotFoundError as e:
        print(e)
        exit(1)

    # Load config
    config = load_mwutil_config(basedir)

    # Load .env from core
    load_core_env(config)

    loaded: dict[str, MWUtilModule] = {}
    for modname in module_names:
        mod = importlib.import_module(f"mwutil.modules.{modname}")

        for attr in dir(mod):
            obj = getattr(mod, attr)
            if isinstance(obj, type) and issubclass(obj, MWUtilModule) and obj is not MWUtilModule:
                loaded[modname] = obj() # instantiate

    config.modules = loaded

    for modname, mod in loaded.items():
        mod_parser = subparsers.add_parser(modname, help=mod.get_description())
        mod.populate_subparser(mod_parser, config)

        mod_parser.set_defaults(func=mod.execute)

    argcomplete.autocomplete(parser)
    args = parser.parse_args()

    if hasattr(args, "func"):
        args.func(config, args)
    else:
        parser.print_help()
