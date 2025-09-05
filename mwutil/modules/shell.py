from argparse import Namespace

from mwutil.module import MWUtilModule


class Shell(MWUtilModule):

    def get_description(self):
        return "Start an interactive PHP shell"

    def populate_subparser(self, parser, config):
        pass

    def execute(self, config, args):
        config.modules["run"].execute(config, Namespace(script="shell", extra_args=[]))
