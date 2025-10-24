import argparse
import importlib
import inspect
import os
from pathlib import Path

import argcomplete

from mwutil.local_config import find_mwutil_config, load_mwutil_config
from mwutil.module import MWUtilModule, GlobalMWUtilModule
from mwutil.utils import load_core_env

GLOBAL_MODULE_NAMES = [
    "security"
]

MODULE_NAMES = [
    "add-gerrit-ssh-key",
    "bash",
    "clone",
    "composer",
    "db",
    "down",
    "dump",
    "elasticsearch",
    "lint",
    "list-repo-remotes",
    "phan",
    "phpunit",
    "recreate",
    "reset",
    "run",
    "setup-gerrit",
    "setup-github",
    "shell",
    "sql",
    "up",
    "update"
] + GLOBAL_MODULE_NAMES

def main():
    parser = argparse.ArgumentParser(description="Manage MediaWiki development environments")

    subparsers = parser.add_subparsers(help="Run a module")

    debug = os.getenv("MWUTIL_DEBUG")

    # Detect if we're in a mediawiki installation
    try:
        basedir: Path = find_mwutil_config()
        if debug:
            print(f"Found .mwutil.json in {basedir}")

        # Load config
        config = load_mwutil_config(basedir)
        load_core_env(config)
        loaded = load_modules(MODULE_NAMES)
        config.modules = loaded
    except FileNotFoundError:
        if debug:
            print(".mwutil.json not found, loading global modules only")

        loaded = load_modules(GLOBAL_MODULE_NAMES)
        config = None

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

def load_modules(module_names: list[str]) -> dict[str, MWUtilModule]:
    loaded: dict[str, MWUtilModule] = {}
    for modname in module_names:
        mod = importlib.import_module(f"mwutil.modules.{modname}")

        for attr in dir(mod):
            obj = getattr(mod, attr)
            if (
                isinstance(obj, type)
                and issubclass(obj, MWUtilModule)
                and obj not in {MWUtilModule, GlobalMWUtilModule}
                and not inspect.isabstract(obj)
            ):
                loaded[modname] = obj()  # instantiate

    return loaded
