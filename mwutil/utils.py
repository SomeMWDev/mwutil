import os
import subprocess
from dataclasses import dataclass
from enum import Enum
from subprocess import CompletedProcess
from typing import Callable

from argcomplete.completers import BaseCompleter
from dotenv import load_dotenv
import json
from pathlib import Path

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
    coredir: Path
    dumpdir: Path
    env: dict = None
    modules: dict = None
    dbtype: DBType = None

def load_mwutil_config(basedir: Path) -> MWUtilConfig:
    file = basedir / ".mwutil.json"
    json_data = json.load(open(file))

    coredir_name = json_data.get("coredir") or "core"
    coredir = basedir / coredir_name
    dumpdir_name = json_data.get("dumpdir") or "dumps"
    dumpdir = basedir / dumpdir_name

    return MWUtilConfig(
        basedir,
        coredir,
        dumpdir
    )

def load_core_env(config: MWUtilConfig):
    env_file = config.coredir / ".env"
    load_dotenv(dotenv_path=env_file)

    config.dbtype = DBType.from_string(os.getenv("MWC_DB_TYPE"))

def run_docker_command(config: MWUtilConfig, command: list[str], capture_output=False, input_text: str | None = None, text: bool = True) -> CompletedProcess:
    cmd = [
        "docker", "compose", "-p", os.getenv("DOCKER_COMPOSE_PROJECT_NAME")
    ] + command

    return subprocess.run(cmd, cwd=config.coredir, capture_output=capture_output, input=input_text, text=text)


def run_container_command(
    config: MWUtilConfig,
    command: list[str],
    container_name: str = "mediawiki",
    exec_options: list[str] | None = None,
    capture_output=False,
    input_text: str | None = None,
    text: bool = True
) -> CompletedProcess:
    return run_docker_command(
        config,
        ["exec"] + (exec_options or []) + [container_name] + command,
        capture_output=capture_output,
        input_text=input_text,
        text=text
    )

def run_command(command: list[str], path: Path | None = None, capture_output=False) -> CompletedProcess:
    return subprocess.run(command, cwd=path, capture_output=capture_output)

def run_db_command(
        config: MWUtilConfig,
        command: list[str] | str,
        options: list[str],
        exec_options: list[str] | None = None,
        capture_output=False,
        input_text: str | None = None,
        text: bool = True
) -> CompletedProcess:
    # TODO move to config
    user = os.getenv("MWC_DB_USER")
    password = os.getenv("MWC_DB_PASSWORD")

    if type(command) is str:
        command = [command]

    return run_container_command(
        config,
        command + [
            f"-u{user}",
            f"-p{password}"
        ] + options,
        config.dbtype.container_name,
        exec_options=exec_options,
        capture_output=capture_output,
        input_text=input_text,
        text=text,
    )

def run_sql_query(
        config: MWUtilConfig,
        query: str
) -> subprocess.CompletedProcess:
    return run_db_command(
        config,
        config.dbtype.get_query_command(),
        [
            "-e",
            query
        ]
    )

def set_git_config(option: str, value: str, folder: Path | None = None) -> CompletedProcess:
    """
    Sets a local git config option.
    The folder this is run in must contain a git repository.
    """
    return run_command(["git", "config", "--local", option, value], folder)

class LazyChoicesCompleter(BaseCompleter):
    def __init__(self, choices_function: Callable):
        self.choices_function = choices_function

    def _convert(self, choice):
        if not isinstance(choice, str):
            choice = str(choice)
        return choice

    def __call__(self, **kwargs):
        return (self._convert(c) for c in self.choices_function())

