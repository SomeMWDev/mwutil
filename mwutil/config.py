import json
import os
from dataclasses import dataclass
from enum import Enum
from pathlib import Path


class DBType(Enum):
    MYSQL = ("mysql", "mysql", "mysql", "mysqldump")
    MARIADB = ("mariadb", "mariadb", "mariadb", "mariadb-dump")

    def __init__(self, value, container, query_command, dump_command):
        self.db_name = value
        self.container_name = container
        self.query_command = query_command
        self.dump_command = dump_command

    def __str__(self):
        return self.db_name

    def get_container(self):
        return self.container_name

    def get_query_command(self):
        return self.query_command

    def get_dump_command(self):
        return self.dump_command

    @classmethod
    def from_string(cls, name: str):
        """Convert string to DBType enum (case-insensitive)."""
        for db in cls:
            if db.db_name.lower() == name.lower():
                return db
        raise ValueError(f"No matching DBType for '{name}'")

@dataclass
class MWUtilConfig:
    basedir: Path
    configdir: Path
    coredir: Path
    dumpdir: Path
    env: dict = None
    modules: dict = None
    dbtype: DBType = None
    mw_install_path: str = None
    mw_branch: str = None

def load_mwutil_config(basedir: Path) -> MWUtilConfig:
    file = basedir / ".mwutil.json"
    json_data = json.load(open(file))

    configdir_name = json_data.get("configdir") or "config"
    configdir = basedir / configdir_name
    coredir_name = json_data.get("coredir") or "core"
    coredir = basedir / coredir_name
    dumpdir_name = json_data.get("dumpdir") or "dumps"
    dumpdir = basedir / dumpdir_name

    return MWUtilConfig(
        basedir,
        configdir,
        coredir,
        dumpdir
    )

def populate_config_from_env(config: MWUtilConfig):
    config.dbtype = DBType.from_string(os.getenv("MWC_DB_TYPE"))
    config.mw_install_path = os.getenv("MW_INSTALL_PATH")
    config.mw_branch = os.getenv("MW_BRANCH") or "master"

def find_mwutil_config(start_path: Path | None = None) -> Path:
    """
    Climb up from start_path (or cwd) until a .mwutil.json file is found.
    Returns the Path to the directory containing it.
    Raises FileNotFoundError if it reaches the root without finding the file.
    """
    current = start_path or Path.cwd()

    while True:
        candidate = current / ".mwutil.json"
        if candidate.is_file():
            return current

        if current.parent == current:
            # reached filesystem root
            raise FileNotFoundError("Could not find .mwutil.json in any parent directory.")

        current = current.parent


