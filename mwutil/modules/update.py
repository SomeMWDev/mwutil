from argparse import Namespace

from mwutil.module import MWUtilModule

class Update(MWUtilModule):

    def get_description(self):
        return "Run update.php"

    def populate_subparser(self, parser, config):
        pass

    def execute(self, config, args):
        config.modules["run"].execute(config, Namespace(script="update", extra_args=["--quick"]))
