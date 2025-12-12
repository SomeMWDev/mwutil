import subprocess
from argparse import ArgumentParser, Namespace
from pathlib import Path

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

        parser.add_argument(
            "-m",
            "--method",
            type=str,
            choices=["cat", "python"],
            default="cat"
        )

    def execute(self, config: MWUtilConfig, args: Namespace):
        file = args.file
        file_path = self.get_file_path(config, file)
        if args.method == "cat":
            subprocess.run(["cat", file_path], check=True)
        elif args.method == "python":
            with open(file_path, "r", encoding="utf-8") as f:
                print(f.read())

    def get_file_path(self, config: MWUtilConfig, file: str) -> Path:
        replacements = {
            "core": config.coredir.absolute()
        }

        template = self.FILES.get(file)
        return Path(template.format(**replacements))
