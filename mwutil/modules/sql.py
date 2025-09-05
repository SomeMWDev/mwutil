from argparse import Namespace

from mwutil.module import MWUtilModule


class SQL(MWUtilModule):

    def get_description(self):
        return "Start an SQL shell"

    def populate_subparser(self, parser, config):
        pass

    def execute(self, config, args):
        config.modules["run"].execute(config, Namespace(script="sql", extra_args=[]))
