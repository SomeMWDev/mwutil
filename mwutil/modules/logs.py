from argparse import ArgumentParser, Namespace

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
            choices=self.FILES.keys()
        )

    def execute(self, config: MWUtilConfig, args: Namespace):
        file = args.file
        template = self.FILES.get(file)
        file = FileWrapper.from_path(config, template)
        file.stream_to_stdout()
