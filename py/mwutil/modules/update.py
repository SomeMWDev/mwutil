from argparse import Namespace, ArgumentParser

from mwutil.local_config import MWUtilConfig
from mwutil.module import MWUtilModule

class Update(MWUtilModule):

    def get_description(self):
        return "Run update.php"

    def populate_subparser(self, parser: ArgumentParser, config: MWUtilConfig):
        pass

    def execute(self, config: MWUtilConfig, args: Namespace):
        config.modules["run"].execute(config, Namespace(script="update", extra_args=["--quick"]))
