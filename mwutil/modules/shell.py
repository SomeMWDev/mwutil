from argparse import Namespace, ArgumentParser

from mwutil.config import MWUtilConfig
from mwutil.module import MWUtilModule


class Shell(MWUtilModule):

    def get_description(self):
        return "Start an interactive PHP shell"

    def populate_subparser(self, parser: ArgumentParser, config: MWUtilConfig):
        pass

    def execute(self, config: MWUtilConfig, args: Namespace):
        config.modules["run"].execute(config, Namespace(script="shell", extra_args=[]))
