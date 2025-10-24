import os
from dataclasses import dataclass
from enum import Enum
from typing import Callable

from mwutil import constants
from mwutil.exec import get_git_option


class EnvOptionMode(Enum):
    BASIC = 1
    ADVANCED = 2
    AUTOMATIC = 3

@dataclass
class EnvOption:
    key: str
    prompt: str
    default: str | Callable[[dict[str, str], str], str] | None = None
    mode: EnvOptionMode = EnvOptionMode.BASIC
    validation_pattern: str | None = None
    reference: str | None = None
    examples: list[str] | None = None
    confidential: bool = False
    allow_empty: bool = False

ENV_OPTIONS = [
        # TODO consider select thingy here
        EnvOption(
            "MW_SCRIPT_PATH",
            "Enter the script path for MediaWiki",
            default="/w",
            mode=EnvOptionMode.ADVANCED,
            reference="https://www.mediawiki.org/wiki/Manual:$wgScriptPath",
        ),
        EnvOption(
            "MW_DOCKER_PORT",
            "Enter the port to expose MediaWiki on your host machine",
            default="8080",
            validation_pattern=r"^\d{2,5}$",
        ),
        EnvOption(
            "MW_SERVER",
            "Enter the server name for MediaWiki",
            default=lambda options, project_name: f"http://localhost:{options['MW_DOCKER_PORT']}",
            mode=EnvOptionMode.ADVANCED,
            reference="https://www.mediawiki.org/wiki/Manual:$wgServer",
        ),
        EnvOption(
            "MW_LANG",
            "Enter the default language code for MediaWiki",
            default="en",
            mode=EnvOptionMode.ADVANCED,
            reference="https://www.mediawiki.org/wiki/Manual:$wgLanguageCode",
        ),
        EnvOption(
            "MW_SITENAME",
            "Enter the site name for MediaWiki",
            default=lambda options, project_name: f"{project_name.capitalize()} Wiki",
            mode=EnvOptionMode.ADVANCED,
            reference="https://www.mediawiki.org/wiki/Manual:$wgSitename",
        ),
        EnvOption(
            "MW_META_NAMESPACE",
            "Enter the meta namespace name for MediaWiki",
            default=lambda options, project_name: options["MW_SITENAME"].replace(" ", "_"),
            mode=EnvOptionMode.ADVANCED,
            reference="https://www.mediawiki.org/wiki/Manual:$wgMetaNamespace",
            validation_pattern=r"^[^ ]+$",
        ),
        EnvOption(
            "MEDIAWIKI_USER",
            "Enter the username for the default MediaWiki admin account",
            default="Admin",
            mode=EnvOptionMode.ADVANCED,
            validation_pattern=constants.LEGAL_TITLE_REGEX,
        ),
        EnvOption(
            "MEDIAWIKI_PASSWORD",
            "Enter the password for the default MediaWiki admin account",
            confidential=True,
            validation_pattern=r"^.{4,}$",
        ),
        EnvOption(
            "MW_INSTALL_PATH",
            "Enter the installation path for MediaWiki inside the container",
            default="/var/www/html/w",
            mode=EnvOptionMode.ADVANCED,
            reference="https://www.mediawiki.org/wiki/Manual:$wgInstallPath",
        ),
        EnvOption(
            "MW_LOG_DIR",
            "",
            default=lambda options, project_name: options["MW_INSTALL_PATH"] + "/cache",
            mode=EnvOptionMode.AUTOMATIC,
        ),
        EnvOption(
            "COMPOSER_CACHE_DIR",
            "",
            default=lambda options, project_name: options["MW_INSTALL_PATH"] + "/cache/composer",
            mode=EnvOptionMode.AUTOMATIC,
        ),
        # TODO select with optional string input?
        EnvOption(
            "MW_BRANCH",
            "Enter the MediaWiki branch to use",
            default="master",
            examples=["master", "REL1_43", "1.45.0-wmf.24"],
        ),
        # TODO xdebug stuff
        EnvOption(
            "DOCKER_COMPOSE_PROJECT_NAME",
            "",
            default=lambda options, project_name: project_name.lower(),
            mode=EnvOptionMode.AUTOMATIC,
        ),
        # TODO use select here
        EnvOption(
            "MWC_DB_TYPE",
            "The type of database to use",
            default="mariadb",
            examples=["mariadb", "mysql"],
            mode=EnvOptionMode.ADVANCED,
            validation_pattern="^(mariadb|mysql)$",
        ),
        EnvOption(
            "MWC_DB_HOST",
            "",
            default=lambda options, project_name: options["MWC_DB_TYPE"],
            mode=EnvOptionMode.AUTOMATIC,
        ),
        EnvOption(
            "MWC_DB_ROOT_PASSWORD",
            "The root password for the database",
            confidential=True,
            default=lambda options, project_name: os.urandom(16).hex(),
            mode=EnvOptionMode.ADVANCED,
            validation_pattern=r"^.{4,}$",
        ),
        EnvOption(
            "MWC_DB_USER",
            "The username for the MediaWiki database user",
            default="mwuser",
            # TODO is this validation correct?
            validation_pattern=r"^[a-zA-Z0-9_]+$",
            mode=EnvOptionMode.ADVANCED,
        ),
        EnvOption(
            "MWC_DB_PASSWORD",
            "The password for the MediaWiki database user",
            confidential=True,
            default=lambda options, project_name: os.urandom(16).hex(),
            mode=EnvOptionMode.ADVANCED,
            validation_pattern=r"^.{4,}$",
        ),
        EnvOption(
            "MWC_DB_DATABASE",
            "The name of the MediaWiki database",
            default=lambda options, project_name: project_name.lower(),
            validation_pattern=r"^[a-zA-Z0-9_\-]+$",
            mode=EnvOptionMode.ADVANCED,
        ),
        EnvOption(
            "MW_SECRET_KEY",
            "The secret key for MediaWiki",
            default=lambda options, project_name: os.urandom(32).hex(),
            mode=EnvOptionMode.ADVANCED,
            reference="https://www.mediawiki.org/wiki/Manual:$wgSecretKey",
            confidential=True,
        ),
        EnvOption(
            "MW_UPGRADE_KEY",
            "The upgrade key for MediaWiki",
            default=lambda options, project_name: os.urandom(16).hex(),
            mode=EnvOptionMode.ADVANCED,
            reference="https://www.mediawiki.org/wiki/Manual:$wgUpgradeKey",
            confidential=True,
        ),
        EnvOption(
            "GIT_USERNAME",
            "Enter your Git username you generally use",
            default=lambda options, project_name: get_git_option("user.name"),
            examples=["YourUsername"],
        ),
        EnvOption(
            "GIT_EMAIL",
            "Enter your Git email you generally use",
            validation_pattern=r"^[^@]+@[^@]+\.[^@]+$",
            default=lambda options, project_name: get_git_option("user.email"),
            examples=["my@example.email"],
        ),
        # TODO SECURITY_PATCH_FOLDER could be questioned in security.py?
        EnvOption(
            "GERRIT_USERNAME",
            "Enter your Gerrit username used to clone repositories via SSH",
            examples=["yourgerritusername"],
        ),
        # TODO GERRIT_SSH_KEY could be questioned somewhere else?
    ]