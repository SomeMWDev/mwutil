import os
import re
import subprocess
from argparse import ArgumentParser, Namespace

import questionary
from rich.console import Console

from mwutil.local_config import MWUtilConfig
from mwutil.module import GlobalMWUtilModule

class Init(GlobalMWUtilModule):

    def get_description(self):
        return "Initialize a new MediaWiki development environment"

    def populate_subparser(self, parser: ArgumentParser, config: MWUtilConfig | None):
        parser.add_argument(
            "-a",
            "--advanced",
            action="store_true",
            help="Offer advanced configuration options",
        )

        parser.add_argument(
            "project_name",
            type=str,
            help="Name of the new MediaWiki project. Can contain only letters, numbers, hyphens and underscores.",
        )

    def execute(self, config: MWUtilConfig | None, args: Namespace):
        debug = os.getenv("MWUTIL_DEBUG")

        console = Console()

        if config is not None:
            print_failure(console, "You are already in a MediaWiki development environment.")
            return

        with console.status("Checking prerequisites...", spinner="dots"):
            self.check_prerequisites(console, debug)
        print_success(console, "All prerequisites met!")

        project_name_pattern = re.compile(r"^[a-zA-Z0-9_-]+$")
        if not project_name_pattern.match(args.project_name):
            print_failure(console, "Invalid project name. It can contain only letters, numbers, hyphens and underscores.")
            exit(1)

        # check if folder exists
        if os.path.exists(args.project_name):
            print_failure(console, f"The folder '{args.project_name}' already exists in the current directory.")
            exit(1)

    @staticmethod
    def check_prerequisites(console: Console, debug: bool | None):
        # check if "docker ps" returns valid output without sudo
        try:
            subprocess.run(
                ["docker", "ps"],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=True,
            )
        except (subprocess.CalledProcessError, FileNotFoundError):
            print_failure(console, "Docker does not seem to be running or is not accessible.")
            print_failure(console, "Please ensure Docker is installed and running, and that you have permission to run it without sudo.")
            print_failure(console, "See https://askubuntu.com/a/477554 for guidance on managing Docker as a non-root user.")
            if debug:
                console.print_exception()
            exit(1)

        # check if git review is available
        try:
            subprocess.run(
                ["git", "review", "--version"],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=True,
            )
        except subprocess.CalledProcessError:
            print_failure(console, "git-review is not installed.")
            print_failure(console, "Please install git-review to proceed. On Fedora, you can install it via 'sudo dnf install git-review'.")
            if debug:
                console.print_exception()
            exit(1)

def print_success(console: Console, message: str):
    console.print(f"[bold green]✔ {message}")

def print_failure(console: Console, message: str):
    console.print(f"[bold red]✖ {message}")

def print_normal(console: Console, message: str):
    console.print(f"[white]{message}")

def print_info(console: Console, message: str):
    console.print(f"[bold blue]ℹ {message}")

def print_detail(console: Console, message: str):
    console.print(f"[bright_black]{message}")
