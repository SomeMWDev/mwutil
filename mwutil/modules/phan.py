from argparse import Namespace
from subprocess import CompletedProcess

from mwutil.module import MWUtilModule
from mwutil.utils import run_container_command


# TODO consider unifying some of this code with lint.py
class Phan(MWUtilModule):

    def get_description(self):
        return "Run phan in a certain directory"

    def populate_subparser(self, parser, config):
        # TODO auto-complete e.g. extensions/Echo
        parser.add_argument(
            "folder",
            type=str,
            default=config.mw_install_path,
            nargs="?"
        )

    def execute(self, config, args):
        def run_phan_command() -> CompletedProcess:
            return run_container_command(config, ["vendor/bin/phan -d . --long-progress-bar"], workdir=args.folder)

        result = run_phan_command()
        if result.returncode == 127:
            print("Failed to run phan. Attempting to update dependencies...")
            config.modules["composer"].execute(config, Namespace(folder=args.folder, extra_args=[]))
            print("Retrying...")
            run_phan_command()
