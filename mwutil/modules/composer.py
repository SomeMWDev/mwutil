import argparse
from argparse import ArgumentParser, Namespace

from mwutil.local_config import MWUtilConfig
from mwutil.exec import run_container_command
from mwutil.module import MWUtilModule


class Composer(MWUtilModule):

    def get_description(self):
        return "Run composer update in core or a specific directory"

    def populate_subparser(self, parser: ArgumentParser, config: MWUtilConfig):
        parser.add_argument(
            "-f",
            "--folder",
            type=str,
            default=config.mw_install_path,
            nargs="?"
        )

        parser.add_argument(
            "extra_args",
            nargs=argparse.REMAINDER,
            help="Additional arguments to pass to composer"
        )
        pass

    def execute(self, config: MWUtilConfig, args: Namespace):
        run_container_command(config, [
            "composer", "update" + " ".join(args.extra_args)
        ], workdir=args.folder)
