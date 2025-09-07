import os
import re
from argparse import Namespace

from mwutil.module import MWUtilModule
from mwutil.utils import run_command, run_container_command


class Pull(MWUtilModule):

    def get_description(self):
        return "Pull an extension or a skin"

    def populate_subparser(self, parser, config):
        parser.add_argument("type", choices=["extension", "skin"], help="Type of repo to pull")

        parser.add_argument("--name", help="Name of extension or skin")

        origin_group = parser.add_mutually_exclusive_group(required=True)
        # example: extensions/UserVerification
        origin_group.add_argument("--gerrit", type=str, help="Pull a repo from gerrit")
        # example: StarCitizenTools/mediawiki-skins-Citizen
        origin_group.add_argument("--github", type=str, help="Pull a repo from Github")

        parser.add_argument("--shallow", "--quick", action='store_true', help="Pull with --depth=1")
        parser.add_argument("--method", type=str, default="ssh", choices=["ssh", "https"],
                            help="The method that should be used to pull the repo")

    def execute(self, config, args):
        name = ""
        origin = ""
        if args.gerrit:
            repo = args.gerrit
            if "/" in repo:
                name = repo.split("/")[-1]
            else:
                name = repo
                repo = f"{args.type}s/{repo}"
            if args.method == "ssh":
                gerrit_username = os.getenv("GERRIT_USERNAME")
                origin = f"ssh://{gerrit_username}@gerrit.wikimedia.org:29418/mediawiki/{repo}"
            elif args.method == "https":
                origin = f"https://gerrit.wikimedia.org/r/mediawiki/{repo}"
        elif args.github:
            if args.method == "ssh":
                origin = f"git@github.com:{args.github}"
            elif args.method == "https":
                origin = f"https://github.com/{args.github}.git"

            name = args.github.split("/")[-1]
            regex = re.compile("mediawiki-(?:extension|skin)s?-(.*)")
            result = re.search(regex, name)
            if result:
                name = result.group(1)
        else:
            print("This should never happen...")
            exit(1)

        if args.name:
            name = args.name

        target_folder_name = args.type + "s"
        target_folder = config.basedir / target_folder_name
        command = [
            "git",
            "clone",
            origin,
            name
        ]
        if args.shallow:
            command.extend(["--depth", "1"])
        run_command(command, target_folder)

        os.chdir(target_folder / name)
        if args.gerrit:
            config.modules["setup-gerrit"].execute(config, Namespace())
        elif args.github:
            config.modules["setup-github"].execute(config, Namespace())

        run_container_command(config, ["composer", "update"], "mediawiki")
        config.modules["update"].execute(config, Namespace())
