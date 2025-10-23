import os
import re
from argparse import Namespace

from mwutil.module import MWUtilModule
from mwutil.utils import run_command


class Clone(MWUtilModule):

    def get_description(self):
        return "Clone an extension or a skin"

    def populate_subparser(self, parser, config):
        parser.add_argument("type", choices=["extension", "skin", "service"], help="Type of repo to pull")

        parser.add_argument("--name", help="Name of extension or skin")

        parser.add_argument("origin", choices=["github", "gerrit"], help="Origin of repo")
        parser.add_argument("repo", help="Repo to pull")

        parser.add_argument("--shallow", "--quick", action='store_true', help="Pull with --depth=1")
        parser.add_argument("--method", type=str, default="ssh", choices=["ssh", "https"],
                            help="The method that should be used to pull the repo")
        parser.add_argument("--composer", action='store_true', help="Run composer update after cloning")
        parser.add_argument("--branch", type=str, help="Branch to clone", default=None)

    def execute(self, config, args):
        name = ""
        origin = ""
        repo = args.repo
        if args.origin == "gerrit":
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
        elif args.origin == "github":
            if args.method == "ssh":
                origin = f"git@github.com:{args.repo}"
            elif args.method == "https":
                origin = f"https://github.com/{args.repo}.git"

            name = repo.split("/")[-1]
            regex = re.compile("mediawiki-(?:extension|skin|service)s?-(.*)")
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

        def clone(shallow: bool, branch: str | None) -> int:
            command = [
                "git",
                "clone",
                origin,
                name
            ]
            if shallow:
                command.extend(["--depth", "1"])
            if branch:
                command.extend(["--branch", branch])
            return run_command(command, target_folder).returncode

        if args.branch:
            clone(args.shallow, args.branch)
        else:
            # try to clone core branch first
            exit_code = clone(args.shallow, config.mw_branch)
            # TODO: validate the git error message here! it looks something like
            # "fatal: Remote branch REL1_44 not found in upstream origin" and is printed to stderr.
            # However, we don't want to capture the output of the git command right now (as it would break the progress
            # bar), so we just rely on the exit code for now.
            if exit_code == 128:
                # fallback to default branch
                print(f"Branch {config.mw_branch} not found, cloning default branch instead.")
                clone(args.shallow, None)

        os.chdir(target_folder / name)
        if args.origin == "gerrit":
            config.modules["setup-gerrit"].execute(config, Namespace())
        elif args.origin == "github":
            config.modules["setup-github"].execute(config, Namespace())

        if args.composer:
            config.modules["composer"].execute(config, Namespace(folder=config.mw_install_path, extra_args=[]))
        if args.type == "extension":
            config.modules["update"].execute(config, Namespace())
