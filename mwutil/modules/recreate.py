from mwutil.module import MWUtilModule
from mwutil.utils import run_docker_command


class Recreate(MWUtilModule):

    def get_description(self):
        return "Recreate all containers"

    def populate_subparser(self, parser, config):
        pass

    def execute(self, config, args):
        run_docker_command(config, ["up", "-d", "--force-recreate"])
