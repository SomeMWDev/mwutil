from argparse import Namespace

from mwutil.module import MWUtilModule

class Install(MWUtilModule):

    def get_description(self):
        return "Install an extension or skin"

    def populate_subparser(self, parser, config):
        pass

    def execute(self, config, args):
        config.modules["run"].execute(config, Namespace(script="update", extra_args=["--quick"]))
