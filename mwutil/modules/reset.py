import os
from argparse import Namespace

from mwutil.module import MWUtilModule
from mwutil.utils import run_sql_query, run_container_command


class Reset(MWUtilModule):

    def get_description(self):
        return "Drop the wiki's database and reinstall it"

    def populate_subparser(self, parser, config):
        pass

    def execute(self, config, args):
        print("Deleting uploads...")
        run_container_command(config, ["rm", "-rf", "/var/www/html/w/images/*"], "mediawiki")

        database = os.getenv("MWC_DB_DATABASE")
        print("Dropping database...")
        run_sql_query(config, f"DROP DATABASE `{database}`;")

        # Move LocalSettings.php temporarily so the installer will run without complaining
        local_settings = config.coredir / "LocalSettings.php"
        tmp_settings = config.coredir / "LocalSettings.temp.php"
        local_settings.rename(tmp_settings)

        # Run the installer
        # TODO move all envs to config
        user = os.getenv("MWC_DB_USER")
        password = os.getenv("MWC_DB_PASSWORD")
        database = os.getenv("MWC_DB_DATABASE")
        dbhost = os.getenv("MWC_DB_HOST")
        config.modules["run"].execute(config, Namespace(script="install", extra_args=[
            f"--dbname={database}",
            f"--dbuser={user}",
            f"--dbpass={password}",
            f"--dbserver={dbhost}",
            f"--server={os.getenv("MW_SERVER")}",
            f"--scriptpath={os.getenv("MW_SCRIPT_PATH")}",
            f"--lang={os.getenv("MW_LANG")}",
            f"--pass={os.getenv("MEDIAWIKI_PASSWORD")}",
            "mediawiki",
            os.getenv("MEDIAWIKI_USER")
        ]))

        # Move LocalSettings.php back
        tmp_settings.rename(local_settings)

        print("Running update.php...")
        config.modules["update"].execute(config, Namespace())

        print("Resetting Elasticsearch...")
        config.modules["elasticsearch"].execute(config, Namespace(action="reset"))

        print("Recreating containers...")
        config.modules["recreate"].execute(config, Namespace())

        print("Done!")
