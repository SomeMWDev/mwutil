import re
import subprocess
from abc import ABC

from mwutil.exec import run_container_command
from mwutil.local_config import MWUtilConfig


def compile_path_template(config: MWUtilConfig, template: str) -> str:
    replacements = {
        "base": config.basedir.absolute(),
        "config": config.configdir.absolute(),
        "core": config.coredir.absolute(),
        "dumps": config.dumpdir.absolute(),
    }

    return template.format(**replacements)

class FileWrapper(ABC):

    @staticmethod
    def from_path(config: MWUtilConfig, path_template: str) -> 'FileWrapper':
        path = compile_path_template(config, path_template)
        pattern = re.compile("^([A-Za-z0-9+.-]+)://")
        match = pattern.match(path)
        if not match:
            return HostFile(path)
        else:
            container_name = match.group(1)
            container_path = path[len(container_name) + 3:]
            return ContainerFile(container_name, container_path, config)

    def read(self):
        raise NotImplementedError("Subclasses must implement this method")

    def stream_to_stdout(self):
        raise NotImplementedError("Subclasses must implement this method")


class HostFile(FileWrapper):
    def __init__(self, path: str):
        self.path = path

    def read(self):
        with open(self.path, 'r') as file:
            return file.read()

    def stream_to_stdout(self):
        subprocess.run(["cat", self.path], check=True)

class ContainerFile(FileWrapper):
    def __init__(self, container_name: str, path: str, config: MWUtilConfig):
        self.container_name = container_name
        self.path = path
        self.config = config

    def read(self):
        result = run_container_command(
            self.config,
            ['cat', self.path],
            container_name=self.container_name,
            capture_output=True,
            text=True,
            exec_options=["-u", "root"]
        )
        if result.returncode != 0:
            raise Exception(f"Failed to read file {self.path} from container {self.container_name}")
        return result.stdout

    def stream_to_stdout(self):
        run_container_command(
            self.config,
            ['cat', self.path],
            container_name=self.container_name,
            capture_output=False,
            text=True,
            exec_options=["-u", "root"]
        )
