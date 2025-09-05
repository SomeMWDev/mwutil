import os
import re
from pathlib import Path

from mwutil.constants import SUPPORTED_BRANCHES
from mwutil.module import MWUtilModule
from mwutil.utils import run_command

class Security(MWUtilModule):

    def get_description(self):
        return "Create and push security patches"

    def populate_subparser(self, parser, config):
        subparsers = parser.add_subparsers(help="The action to perform", dest="action")

        create_patch_parser = subparsers.add_parser("create-patch")
        create_patch_parser.add_argument(
            "name",
            type=str,
            help="The name of the patch to create",
            default="",
            nargs="?"
        )
        create_patch_parser.add_argument(
            "--use-branch-name",
            action="store_true",
            help="Use the branch name as the patch name",
            default=False
        )

        subparsers.add_parser("push-all")

    def execute(self, config, args):
        if args.action == "create-patch":
            security_patch_dir = Path(os.getenv("SECURITY_PATCH_FOLDER"))
            if not security_patch_dir.exists():
                print("Creating security patch directory...")
                security_patch_dir.mkdir()
                print("Created security patch directory.")

            name = args.name
            if name == "":
                # retrieve branch name
                branch_name: str = run_command(
                    [
                        "git",
                        "branch",
                        "--show-current"
                    ],
                    None,
                    True
                ).stdout.decode().strip()
                print(f"No patch name provided. Current branch: {branch_name}")
                if args.use_branch_name:
                    name = branch_name
                    print(f"Using branch name: {name}")
                elif re.search("^T[0-9]{4,10}$", branch_name):
                    name = branch_name
                    print(f"Using branch name, since a task ID was detected in it: {name}")
                else:
                    print("Please specify a patch name, or use the --use-branch-name option")
                    exit(1)

            # create patch
            patch_file = security_patch_dir / f"{name}.patch"
            run_command(
                [
                    "git",
                    "format-patch",
                    "HEAD^",
                    "--output",
                    patch_file,
                ]
            )
        elif args.action == "push-all":
            print("Checking for unpushed SECURITY commits...")

            commits_to_push: dict[str, list[str]] = {}

            for branch in SUPPORTED_BRANCHES:
                print(f"Checking branch {branch}...")

                run_command(["git", "fetch", "origin", branch])
                run_command(["git", "checkout", branch])

                commits_raw = run_command([
                    "git",
                    "log",
                    f"origin/{branch}..HEAD",
                    "--grep=^SECURITY:",
                    "--pretty=format:%h"
                ], None, True).stdout.decode().strip()

                if commits_raw:
                    commits = commits_raw.splitlines()
                    commits_to_push[branch] = commits
                    print(f"  {branch}:")
                    for commit in commits:
                        msg = run_command([
                            "git",
                            "log",
                            "-1",
                            "--pretty=format:%s",
                            commit
                        ], None, True).stdout.decode().strip()
                        print(f"    - {msg}")

            if not commits_to_push:
                print("No SECURITY commits to push on any branch.")
                return

            print()
            confirm = input("Push SECURITY commits to the above branches? (y/N): ")
            if confirm.lower() != "y":
                print("Aborted.")
                return

            print("Pushing commits...")
            for branch, commits in commits_to_push.items():
                print(f"Pushing {branch}...")
                run_command(["git", "checkout", branch])
                run_command(["git", "push", "origin", f"HEAD:refs/for/{branch}"])

            print("Done.")

        else:
            print("Unknown action")
            exit(1)
