from abc import ABC, abstractmethod
from argparse import ArgumentParser, Namespace

from mwutil.utils import MWUtilConfig


class MWUtilModule(ABC):

    @abstractmethod
    def get_description(self) -> str:
        pass

    @abstractmethod
    def populate_subparser(self, parser: ArgumentParser, config: MWUtilConfig):
        pass

    @abstractmethod
    def execute(self, config: MWUtilConfig, args: Namespace):
        pass

class GlobalMWUtilModule(MWUtilModule):

    @abstractmethod
    def populate_subparser(self, parser: ArgumentParser, config: MWUtilConfig | None):
        pass

    @abstractmethod
    def execute(self, config: MWUtilConfig | None, args: Namespace):
        pass
