import os
from argparse import ArgumentParser, Namespace

from mwutil.module import MWUtilModule
from mwutil.utils import MWUtilConfig, set_git_config


class SetupGithub(MWUtilModule):

    def get_description(self) -> str:
        return "Sets up a local repository that was cloned from Github"

    def populate_subparser(self, parser: ArgumentParser, basedir: MWUtilConfig):
        pass

    def execute(self, config: MWUtilConfig, args: Namespace):
        git_email = os.getenv("GIT_EMAIL")
        git_username = os.getenv("GIT_USERNAME")
        
        set_git_config("user.email", git_email)
        set_git_config("user.name", git_username)

        print("Done!")
