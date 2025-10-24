from mwutil.exec import run_docker_command
from mwutil.module import MWUtilModule


class Recreate(MWUtilModule):

    def get_description(self):
        return "Recreate all containers"

    def populate_subparser(self, parser, config):
        pass

    def execute(self, config, args):
        run_docker_command(config, ["up", "-d", "--force-recreate"])
