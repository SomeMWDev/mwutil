import argparse
from argparse import ArgumentParser, Namespace

from mwutil.config import MWUtilConfig
from mwutil.exec import run_container_command, run_docker_command
from mwutil.module import MWUtilModule
from mwutil.utils import LazyChoicesCompleter


class Bash(MWUtilModule):

    def get_description(self):
        return "Start a bash shell in a container"

    def populate_subparser(self, parser: ArgumentParser, config: MWUtilConfig):
        def get_containers():
            return run_docker_command(
                config,
                ["ps", "--services"],
                capture_output=True
            ).stdout.splitlines()

        parser.add_argument(
            "-c",
            "--container",
            type=str,
            help="The container name",
            default="mediawiki"
        ).completer = LazyChoicesCompleter(get_containers)

        parser.add_argument(
            "-f",
            "--folder",
            type=str,
            default=config.mw_install_path,
            nargs="?"
        )

        parser.add_argument(
            "-r",
            "--root",
            action="store_true",
            help="Execute the command as a root user in the container"
        )

        parser.add_argument(
            "command",
            nargs=argparse.REMAINDER,
            help="Additional arguments to pass to PHPUnit"
        )

    def execute(self, config: MWUtilConfig, args: Namespace):
        command = args.command if args.command else ["bash"]
        exec_options = ["-u", "root"] if args.root else []
        run_container_command(config, command, args.container, exec_options=exec_options, workdir=args.folder)
