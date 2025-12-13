from argparse import ArgumentParser, Namespace

from mwutil.local_config import MWUtilConfig
from mwutil.exec import run_docker_command
from mwutil.module import MWUtilModule


class Recreate(MWUtilModule):

    def get_description(self):
        return "Recreate containers"

    def populate_subparser(self, parser: ArgumentParser, config: MWUtilConfig):
        parser.add_argument(
            "container",
            type=str,
            nargs="?",
            default=None,
            help="Name of the container",
        )

    def execute(self, config: MWUtilConfig, args: Namespace):
        command = ["up", "-d", "--force-recreate"]
        if args.container:
            command.append(args.container)
        run_docker_command(config, command)
