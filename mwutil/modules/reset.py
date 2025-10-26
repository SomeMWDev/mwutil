import os
from argparse import Namespace, ArgumentParser

from mwutil.local_config import MWUtilConfig
from mwutil.exec import run_sql_query, run_container_command
from mwutil.module import MWUtilModule


class Reset(MWUtilModule):

    def get_description(self):
        return "Drop the wiki's database and reinstall it"

    def populate_subparser(self, parser: ArgumentParser, config: MWUtilConfig):
        pass

    def execute(self, config: MWUtilConfig, args: Namespace):
        print("Making sure containers are up...")
        config.modules["up"].execute(config, Namespace())

        print("Deleting uploads...")
        run_container_command(
            config,
            ["find", "images", "-mindepth", "1", "!", "-name", "README", "!", "-name", ".htaccess", "-exec", "rm", "-rf", "{}", "+"],
            "mediawiki"
        )

        database = os.getenv("MWC_DB_DATABASE")
        print("Dropping database...")
        run_sql_query(config, f"DROP DATABASE `{database}`;")

        # Move LocalSettings.php temporarily so the installer will run without complaining
        local_settings = config.coredir / "LocalSettings.php"
        tmp_settings = config.coredir / "LocalSettings.temp.php"
        local_settings.rename(tmp_settings)

        try:
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
        finally:
            # Move LocalSettings.php back
            tmp_settings.rename(local_settings)

        print("Running update.php...")
        config.modules["update"].execute(config, Namespace())

        print("Resetting Elasticsearch...")
        config.modules["elasticsearch"].execute(config, Namespace(action="reset"))

        print("Recreating containers...")
        config.modules["recreate"].execute(config, Namespace())

        print("Done!")
