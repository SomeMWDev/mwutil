import os
from argparse import ArgumentParser, Namespace
from pathlib import Path

from mwutil.local_config import MWUtilConfig
from mwutil.exec import run_wiki_db_command, run_sql_query
from mwutil.module import MWUtilModule
from mwutil.utils import LazyChoicesCompleter


class Dump(MWUtilModule):

    def get_description(self):
        return "Manage database dumps"

    def populate_subparser(self, parser: ArgumentParser, config: MWUtilConfig):
        parser.add_argument("action", type=str, choices=["create", "delete", "import"])

        def get_dumps():
            dumps = []
            for file in config.dumpdir.iterdir():
                file: Path
                if file.suffix == ".sql":
                    dumps.append(file.stem)
            return dumps

        parser.add_argument("dumpname", type=str).completer = LazyChoicesCompleter(get_dumps)

    def execute(self, config: MWUtilConfig, args: Namespace):
        if not config.dumpdir.exists():
            print("Creating dump directory...")
            config.dumpdir.mkdir()
            print("Created dump directory.")

        dump = config.dumpdir / f"{args.dumpname}.sql"

        if args.action == "delete":
            dump.unlink()
            print("Successfully deleted dump file.")
            return

        database = os.getenv("MWC_DB_DATABASE")

        if args.action == "create":
            print("Creating dump...")
            dump_text = run_wiki_db_command(
                config,
                config.dbtype.dump_command,
                [
                    database
                ],
                capture_output=True,
                text=False # it breaks otherwise
            ).stdout.decode('latin1')
            dump.touch()
            dump.write_text(dump_text)
            print(f"Dump created: {dump.absolute()}")
        elif args.action == "import":
            print("Creating database...")
            run_sql_query(config, f"CREATE DATABASE IF NOT EXISTS `{database}`;")
            print("Importing dump...")
            with open(dump, "r") as f:
                run_wiki_db_command(
                    config,
                    config.dbtype.query_command,
                    [
                        database
                    ],
                    exec_options=["-T"],
                    capture_output=False,
                    input_text=f.read()
                )

        else:
            print("Unknown action")
            exit(1)
