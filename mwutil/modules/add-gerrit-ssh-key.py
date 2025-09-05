import os
from argparse import ArgumentParser, Namespace

from mwutil.module import MWUtilModule
from mwutil.utils import MWUtilConfig, run_command


class AddGerritSSHKey(MWUtilModule):

    def get_description(self) -> str:
        return "Add the Gerrit SSH key to the SSH agent"

    def populate_subparser(self, parser: ArgumentParser, basedir: MWUtilConfig):
        pass

    def execute(self, config: MWUtilConfig, args: Namespace):
        run_command(["ssh-add", os.getenv("GERRIT_SSH_KEY")])
