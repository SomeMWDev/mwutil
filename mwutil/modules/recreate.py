from argparse import ArgumentParser, Namespace

from mwutil.config import MWUtilConfig
from mwutil.exec import run_docker_command
from mwutil.module import MWUtilModule


class Recreate(MWUtilModule):

    def get_description(self):
        return "Recreate all containers"

    def populate_subparser(self, parser: ArgumentParser, config: MWUtilConfig):
        pass

    def execute(self, config: MWUtilConfig, args: Namespace):
        run_docker_command(config, ["up", "-d", "--force-recreate"])
