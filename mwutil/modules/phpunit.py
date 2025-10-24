import argparse
from argparse import ArgumentParser, Namespace

from mwutil.local_config import MWUtilConfig
from mwutil.exec import run_container_command
from mwutil.module import MWUtilModule


class PhpUnit(MWUtilModule):

    def get_description(self):
        return "Run tests with PHPUnit"

    def populate_subparser(self, parser: ArgumentParser, config: MWUtilConfig):
        parser.add_argument(
            "extra_args",
            nargs=argparse.REMAINDER,
            help="Additional arguments to pass to PHPUnit"
        )
        pass

    def execute(self, config: MWUtilConfig, args: Namespace):
        run_container_command(config, ["composer", "phpunit:entrypoint", "--"] + args.extra_args)
