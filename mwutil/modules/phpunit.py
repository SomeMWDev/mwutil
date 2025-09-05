import argparse

from mwutil.module import MWUtilModule
from mwutil.utils import run_container_command


class PhpUnit(MWUtilModule):

    def get_description(self):
        return "Run tests with PHPUnit"

    def populate_subparser(self, parser, config):
        parser.add_argument(
            "extra_args",
            nargs=argparse.REMAINDER,
            help="Additional arguments to pass to PHPUnit"
        )
        pass

    def execute(self, config, args):
        run_container_command(config, ["composer", "phpunit:entrypoint", "--"] + args.extra_args)
