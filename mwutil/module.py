from abc import ABC, abstractmethod
from argparse import ArgumentParser, Namespace

from mwutil.utils import MWUtilConfig


class MWUtilModule(ABC):

    @abstractmethod
    def get_description(self) -> str:
        pass

    @abstractmethod
    def populate_subparser(self, parser: ArgumentParser, basedir: MWUtilConfig):
        pass

    @abstractmethod
    def execute(self, config: MWUtilConfig, args: Namespace):
        pass
