from argparse import Namespace, ArgumentParser

from mwutil.config import MWUtilConfig
from mwutil.data_state import disable_profile, enable_profile
from mwutil.exec import run_container_command, run_docker_command
from mwutil.module import MWUtilModule


class ElasticSearch(MWUtilModule):

    def get_description(self):
        return "Manage the local elasticsearch installation"

    def populate_subparser(self, parser: ArgumentParser, config: MWUtilConfig):
        # Todo either add more stuff here or convert to argument
        subparsers = parser.add_subparsers(help="The action to perform", dest="action")

        subparsers.add_parser("reset", help="Reset the elasticsearch index and reindex the wiki")
        subparsers.add_parser("disable", help="Disable the elasticsearch container")
        subparsers.add_parser("enable", help="Enable the elasticsearch container")

    def execute(self, config: MWUtilConfig, args: Namespace):
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
        elif args.action == "disable":
            run_docker_command(config, ["down", "elasticsearch"])
            disable_profile(config, "elasticsearch")
        elif args.action == "enable":
            enable_profile(config, "elasticsearch")
            config.modules["up"].execute(config, Namespace())
        else:
            print("Unknown action")
            exit(1)
