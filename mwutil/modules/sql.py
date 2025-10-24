import argparse
from argparse import Namespace, ArgumentParser
from os import getenv

from mwutil.config import MWUtilConfig
from mwutil.exec import run_root_db_command
from mwutil.module import MWUtilModule


class SQL(MWUtilModule):

    def get_description(self):
        return "Start an SQL shell"

    def populate_subparser(self, parser: ArgumentParser, config: MWUtilConfig):
        parser.add_argument("--root", action="store_true", help="Log in as root user")
        parser.add_argument(
            "extra_args",
            nargs=argparse.REMAINDER,
            help="Additional arguments to pass to the SQL command",
        )

    def execute(self, config: MWUtilConfig, args: Namespace):
        if args.root:
            db = getenv("MWC_DB_DATABASE")
            run_root_db_command(
                config,
                config.dbtype.query_command,
                [
                    db
                ] + args.extra_args,
            )
        else:
            config.modules["run"].execute(config, Namespace(script="sql", extra_args=args.extra_args))
