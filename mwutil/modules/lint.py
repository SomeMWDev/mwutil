from argparse import Namespace, ArgumentParser
from subprocess import CompletedProcess

from mwutil.config import MWUtilConfig
from mwutil.exec import run_container_command
from mwutil.module import MWUtilModule


class Lint(MWUtilModule):

    def get_description(self):
        return "Lint the code in a certain directory"

    def populate_subparser(self, parser: ArgumentParser, config: MWUtilConfig):
        # TODO auto-complete e.g. extensions/Echo
        parser.add_argument(
            "folder",
            type=str,
            default=config.mw_install_path,
            nargs="?"
        )

    def execute(self, config: MWUtilConfig, args: Namespace):
        def run_lint_command() -> CompletedProcess:
            return run_container_command(config, [
                "composer run test"
            ], workdir=args.folder)

        result = run_lint_command()
        if result.returncode == 127:
            print("Failed to lint. Attempting to update dependencies...")
            config.modules["composer"].execute(config, Namespace(folder=args.folder, extra_args=[]))
            print("Retrying...")
            run_lint_command()
