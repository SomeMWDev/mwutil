import time
import uuid
from argparse import Namespace

from mwutil.module import MWUtilModule
from mwutil.utils import disable_profile, enable_profile, set_env_key


class DB(MWUtilModule):

    def get_description(self):
        return "Manage the DB for MediaWiki"

    def populate_subparser(self, parser, config):
        subparsers = parser.add_subparsers(help="The action to perform", dest="action")

        switch_parser = subparsers.add_parser("switch")
        switch_parser.add_argument("db", choices=["mysql", "mariadb"], help="The database to switch to")

    def execute(self, config, args):
        if args.action == "switch":
            dbtype = args.db
            if config.dbtype.db_name == dbtype:
                print(f"Already using {dbtype} profile.")
                exit(0)

            dump_name = uuid.uuid4()
            print("Creating DB dump...")
            config.modules["dump"].execute(config, Namespace(action="create", dumpname=dump_name))

            other_dbtype = "mariadb" if dbtype == "mysql" else "mysql"
            disable_profile(config, other_dbtype)
            enable_profile(config, dbtype)
            print(f"Switched to {dbtype} profile.")
            set_env_key(config, "MWC_DB_TYPE", dbtype)
            set_env_key(config, "MWC_DB_HOST", dbtype)
            print("Restarting containers...")
            config.modules["recreate"].execute(config, Namespace())

            print("Waiting 10 seconds for the database to start...")
            time.sleep(10)

            print("Importing dump...")
            config.modules["dump"].execute(config, Namespace(action="import", dumpname=dump_name))
            print("Imported dump.")

            print("Deleting temporary dump...")
            config.modules["dump"].execute(config, Namespace(action="delete", dumpname=dump_name))
            print("Deleted temporary dump.")
        else:
            print("Unknown action")
            exit(1)
