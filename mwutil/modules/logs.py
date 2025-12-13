from argparse import ArgumentParser, Namespace

from argcomplete import ChoicesCompleter
from hfilesize import FileSize

from mwutil.files import FileWrapper
from mwutil.local_config import MWUtilConfig
from mwutil.module import MWUtilModule


class Logs(MWUtilModule):

    FILES = {
        "dberror": "{core}/cache/mw-dberror.log",
        "debug-cli": "{core}/cache/mw-debug-cli.log",
        "debug-web": "{core}/cache/mw-debug-web.log",
        "error": "{core}/cache/mw-error.log",
        "ratelimit": "{core}/cache/mw-ratelimit.log",
    }

    def get_description(self) -> str:
        return "Manage and print various logs, e.g. for use with grep/less"

    def populate_subparser(self, parser: ArgumentParser, config: MWUtilConfig):
        parser.add_argument(
            "file",
            type=str,
            help="One of the options (e.g. dberror), or a custom file path (e.g. mariadb:///var/log/bootstrap.log)"
        ).completer = ChoicesCompleter(self.FILES.keys())

        action_subparsers = parser.add_subparsers(
            dest="action",
            required=False,
            help="The action to perform on the log file (default: show)"
        )

        action_subparsers.add_parser("show", help="Show the contents of the log file")

        action_subparsers.add_parser("clear", help="Clear the contents of the log file")

        trim_parser = action_subparsers.add_parser("trim", help="Trim the log file")
        trim_parser.add_argument(
            "size",
            type=str,
            help="The size to trim the log file to (e.g. 10M for 10 megabytes)"
        )

    def execute(self, config: MWUtilConfig, args: Namespace):
        file = args.file
        template = self.FILES.get(file) or file
        file = FileWrapper.from_path(config, template)

        action = args.action or "show"
        if action == "show":
            file.stream_to_stdout()
        elif action == "clear":
            print(f"Clearing log file: {template}")
            file.write_text("")
            print("Done.")
        elif action == "trim":
            size = args.size
            max_bytes = FileSize(size)
            text = file.read()
            if len(text) <= max_bytes:
                print("Log file is already within the specified size.")
                return
            prefix = "[... trimmed by mwutil ...]\n"
            trimmed_text = prefix + text[-(max_bytes - len(prefix)):]
            file.write_text(trimmed_text)
            print(f"Trimmed log file to the last {size}.")
