from mwutil.constants import REPO_TYPES
from mwutil.exec import run_command
from mwutil.module import MWUtilModule

class Pull(MWUtilModule):

    def get_description(self):
        return "Pull a local repository"

    def populate_subparser(self, parser, config):
        parser.add_argument(
            "type",
            choices=(REPO_TYPES + ["config", "core"]),
            help="Type of repo to pull"
        )
        parser.add_argument(
            "name",
            help="Name of local repo",
            nargs="?"
        )

    def execute(self, config, args):
        if args.type == "core":
            directory = config.coredir
        elif args.type == "config":
            directory = config.basedir
        else:
            if not args.name:
                print("You must specify a name for this type of repository.")
                exit(1)
            directory = f"{config.basedir}/{args.type}s/{args.name}"
        command = [
            "git", "-C", directory, "pull"
        ]
        run_command(command)
