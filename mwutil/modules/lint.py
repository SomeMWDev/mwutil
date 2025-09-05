from pathlib import Path
from subprocess import CompletedProcess

from mwutil.module import MWUtilModule
from mwutil.utils import run_container_command


class Lint(MWUtilModule):

    def get_description(self):
        return "Lint the code in a certain directory"

    def populate_subparser(self, parser, config):
        # TODO auto-complete e.g. extensions/Echo
        parser.add_argument(
            "folder",
            type=Path,
            default="/var/www/html/w",
            nargs="?"
        )

    def execute(self, config, args):
        def run_lint_command() -> CompletedProcess:
            return run_container_command(config, [
                "bash",
                "-c",
                f"cd '{args.folder}' && composer run test"
            ])

        result = run_lint_command()
        if result.returncode == 127:
            print("Failed to lint. Attempting to update dependencies...")
            run_container_command(config, [
                "bash",
                "-c",
                f"cd '{args.folder}' && composer update"
            ])
            print("Retrying...")
            run_lint_command()
