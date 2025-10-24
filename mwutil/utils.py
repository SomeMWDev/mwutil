import re
from dataclasses import dataclass
from typing import Callable

import dotenv
from argcomplete.completers import BaseCompleter
from dotenv import load_dotenv

from mwutil.config import populate_config_from_env, MWUtilConfig


def load_core_env(config: MWUtilConfig):
    env_file = config.configdir / ".env"
    load_dotenv(dotenv_path=env_file)
    populate_config_from_env(config)

def set_env_key(config: MWUtilConfig, key: str, value: str):
    env_file = config.configdir / ".env"
    dotenv.set_key(env_file, key, value)

class LazyChoicesCompleter(BaseCompleter):
    def __init__(self, choices_function: Callable):
        self.choices_function = choices_function

    def _convert(self, choice):
        if not isinstance(choice, str):
            choice = str(choice)
        return choice

    def __call__(self, **kwargs):
        return (self._convert(c) for c in self.choices_function())

@dataclass
class MWVersion:
    major: int
    minor: int
    patch: int
    suffix: str | None = None

    def __str__(self):
        version = f"{self.major}.{self.minor}.{self.patch}"
        if self.suffix:
            version += f"-{self.suffix}"
        return version

    @staticmethod
    def parse(version_str: str) -> 'MWVersion':
        match = re.match(r"(\d+)\.(\d+)\.(\d+)(?:-([a-zA-Z0-9]+))?", version_str)
        if not match:
            raise ValueError(f"Invalid version string: {version_str}")
        major, minor, patch, suffix = match.groups()
        return MWVersion(int(major), int(minor), int(patch), suffix)

def get_core_version(config: MWUtilConfig) -> MWVersion | None:
    version_file = config.coredir / "includes" / "Defines.php"
    if not version_file.is_file():
        return None

    regex = re.compile(r"'MW_VERSION', '([a-zA-Z0-9\-.]+)'")
    with version_file.open("r", encoding="utf-8") as f:
        for line in f:
            match = regex.search(line)
            if match:
                return MWVersion.parse(match.group(1))
    return None
