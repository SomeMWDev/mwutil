from mwutil.module import MWUtilModule
from mwutil.utils import run_docker_command


class Down(MWUtilModule):

    def get_description(self):
        return "Stop all containers"

    def populate_subparser(self, parser, config):
        pass

    def execute(self, config, args):
        run_docker_command(config, ["down"])
