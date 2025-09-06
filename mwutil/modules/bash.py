import argparse

from mwutil.module import MWUtilModule
from mwutil.utils import run_container_command, run_docker_command, LazyChoicesCompleter


class Bash(MWUtilModule):

    def get_description(self):
        return "Start a bash shell in a container"

    def populate_subparser(self, parser, config):
        def get_containers():
            return run_docker_command(
                config,
                ["ps", "--services"],
                capture_output=True
            ).stdout.splitlines()

        parser.add_argument(
            '-c',
            '--container',
            type=str,
            help="The container name",
            default="mediawiki"
        ).completer = LazyChoicesCompleter(get_containers)

        parser.add_argument(
            "command",
            nargs=argparse.REMAINDER,
            help="Additional arguments to pass to PHPUnit"
        )

    def execute(self, config, args):
        command = args.command if args.command else ["bash"]
        run_container_command(config, command, args.container)
