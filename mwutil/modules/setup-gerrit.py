import os
from argparse import ArgumentParser, Namespace

from mwutil.module import MWUtilModule
from mwutil.utils import MWUtilConfig, run_command, set_git_config


class SetupGerrit(MWUtilModule):

    def get_description(self) -> str:
        return "Sets up a local repository that was cloned from gerrit"

    def populate_subparser(self, parser: ArgumentParser, basedir: MWUtilConfig):
        pass

    def execute(self, config: MWUtilConfig, args: Namespace):
        git_email = os.getenv("GIT_EMAIL")
        git_username = os.getenv("GIT_USERNAME")
        gerrit_username = os.getenv("GERRIT_USERNAME")

        set_git_config("user.email", git_email)
        set_git_config("user.name", git_username)
        set_git_config(f"url.\"ssh://{gerrit_username}@gerrit.wikimedia.org:29418/\".insteadOf", "\"https://gerrit.wikimedia.org/r/\"")
        set_git_config("gitreview.username", git_username)
        set_git_config("gitreview.remote", "origin")

        run_command(["git", "review", "-s", "--verbose"])

        print("Done!")
