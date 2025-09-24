import argparse
from pathlib import Path

from mwutil.module import MWUtilModule
from mwutil.utils import run_container_command


class Composer(MWUtilModule):

    def get_description(self):
        return "Run composer update in core or a specific directory"

    def populate_subparser(self, parser, config):
        parser.add_argument(
            "folder",
            type=Path,
            default="/var/www/html/w",
            nargs="?"
        )

        parser.add_argument(
            "extra_args",
            nargs=argparse.REMAINDER,
            help="Additional arguments to pass to composer"
        )
        pass

    def execute(self, config, args):
        run_container_command(config, [
            "bash",
            "-c",
            f"cd '{args.folder}' && composer update " + " ".join(args.extra_args)
        ])
