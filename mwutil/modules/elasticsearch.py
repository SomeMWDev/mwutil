from argparse import Namespace

from mwutil.module import MWUtilModule
from mwutil.utils import run_container_command


class ElasticSearch(MWUtilModule):

    def get_description(self):
        return "Manage the local elasticsearch installation"

    def populate_subparser(self, parser, config):
        # Todo either add more stuff here or convert to argument
        subparsers = parser.add_subparsers(help="The action to perform", dest="action")

        subparsers.add_parser("reset")

    def execute(self, config, args):
        if args.action == "reset":
            run_container_command(
                config,
                [
                    "curl",
                    "-X",
                    "DELETE",
                    "localhost:9200/_all",
                ],
                "elasticsearch"
            )
            config.modules["run"].execute(config, Namespace(script='CirrusSearch:UpdateSearchIndexConfig', extra_args=['--startOver']))
            config.modules["run"].execute(config, Namespace(script='CirrusSearch:ForceSearchIndex', extra_args=[]))
        else:
            print("Unknown action")
            exit(1)
