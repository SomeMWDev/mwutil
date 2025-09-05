import argparse
from pathlib import Path

from mwutil.module import MWUtilModule
from mwutil.utils import run_container_command, LazyChoicesCompleter


def _get_scripts(coredir):
    scripts = []
    scripts_folder = coredir / "maintenance"

    file: Path
    for file in scripts_folder.iterdir():
        if file.is_file() and file.suffix == ".php":
            scripts.append(file.stem)

    for extension_folder in (coredir / "extensions").iterdir():
        maintenance_folder = extension_folder / "maintenance"
        if not maintenance_folder.is_dir():
            continue
        for file in maintenance_folder.iterdir():
            if file.is_file() and file.suffix == ".php":
                scripts.append(extension_folder.stem + ":" + file.stem)

    return scripts


class Run(MWUtilModule):

    def get_description(self):
        return "Run a maintenance script"

    def populate_subparser(self, parser, config):
        def get_scripts():
            return _get_scripts(config.coredir)

        parser.add_argument(
            "script",
            type=str,
            help="The name of the maintenance script"
        ).completer = LazyChoicesCompleter(get_scripts)
        parser.add_argument(
            "extra_args",
            nargs=argparse.REMAINDER,
            help="Additional arguments to pass to the maintenance script",
        )

    def execute(self, config, args):
        run_container_command(config, ["maintenance/run", args.script] + args.extra_args)
