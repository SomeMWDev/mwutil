import os

from argcomplete import ChoicesCompleter

from mwutil.module import MWUtilModule
from mwutil.utils import run_container_command, run_command


class Bash(MWUtilModule):

    def get_description(self):
        return "Start a bash shell in a container"

    def populate_subparser(self, parser, config):
        containers = []

        if os.getenv("_ARGCOMPLETE") == "1":
            containers = run_command(
                ["docker", "compose", "ps", "--services"],
                config.coredir,
                True
            ).stdout.decode().splitlines()

        parser.add_argument(
            '-c',
            '--container',
            type=str,
            help="The container name",
            default="mediawiki"
        ).completer = ChoicesCompleter(containers)

    def execute(self, config, args):
        run_container_command(config, ["bash"], args.container)
