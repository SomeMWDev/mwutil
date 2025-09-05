from collections import OrderedDict

from mwutil.module import MWUtilModule
from mwutil.utils import run_command


class ListRepoRemotes(MWUtilModule):

    def get_description(self):
        return "List the remotes of all repos in the basedir"

    def populate_subparser(self, parser, config):
        pass

    def execute(self, config, args):
        repo_remotes = {}
        for folder in config.basedir.iterdir():
            if not folder.is_dir():
                # not a folder
                continue
            if not (folder / ".git").exists():
                # not a git repo
                continue
            remote_url = run_command(
                [
                    "git",
                    "config",
                    "--get",
                    "remote.origin.url"
                ],
                folder,
                True
            ).stdout.decode().strip()
            if not remote_url:
                continue
            repo_remotes[folder.name] = remote_url
        repo_remotes = OrderedDict(
            # sort in lower case so "core" is between "CirrusSearch" and "DataMaps"
            sorted(repo_remotes.items(), key=lambda kv: kv[0].lower())
        )
        for repo, remote in repo_remotes.items():
            print(f"{repo}: {remote}")
