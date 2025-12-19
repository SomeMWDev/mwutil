import argparse
from argparse import ArgumentParser, Namespace

from mwutil.local_config import MWUtilConfig
from mwutil.exec import run_container_command
from mwutil.module import MWUtilModule


class ParserTests(MWUtilModule):

    def get_description(self):
        return "Run parser tests"

    def populate_subparser(self, parser: ArgumentParser, config: MWUtilConfig):
        parser.add_argument(
            "-f",
            "--file",
            dest="file",
            help="Parser test file to run",
            type=str
        )

        parser.add_argument(
            "extra_args",
            nargs=argparse.REMAINDER,
            help="Additional arguments to pass to parserTests.php"
        )
        pass

    def execute(self, config: MWUtilConfig, args: Namespace):
        extra_args = args.extra_args
        if args.file:
            extra_args += ["--file", args.file]

        run_container_command(
            config,
            ["php", "tests/parser/parserTests.php"] + extra_args
        )
