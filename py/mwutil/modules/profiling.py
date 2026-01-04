import time
from argparse import Namespace, ArgumentParser

from watchdog.observers import Observer

from mwutil.local_config import MWUtilConfig
from mwutil.module import MWUtilModule
from mwutil.utils import CopyOnChangeHandler


class Profiling(MWUtilModule):

    def get_description(self):
        return "Utilities for profiling MediaWiki requests"

    def populate_subparser(self, parser: ArgumentParser, config: MWUtilConfig):
        subparsers = parser.add_subparsers(help="The action to perform", dest="action")

        subparsers.add_parser("watch", help="Watch the speedscope file and copy it to the clipboard if it changes")

    def execute(self, config: MWUtilConfig, args: Namespace):
        if args.action == "watch":
            file = config.coredir / "cache" / "speedscope.json"
            event_handler = CopyOnChangeHandler(file)
            observer = Observer()
            observer.schedule(event_handler, file.parent, recursive=False)
            observer.start()

            print(f"Watching for changes to {file.resolve()}")
            try:
                while True:
                    time.sleep(1)
            except KeyboardInterrupt:
                observer.stop()
            observer.join()
        else:
            print("Unknown action")
            exit(1)
