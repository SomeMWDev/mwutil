import os
import re
import subprocess
from argparse import ArgumentParser, Namespace
from dataclasses import dataclass
from enum import Enum
from typing import Callable

import dotenv
import questionary
from rich.console import Console
from rich.panel import Panel
from rich.table import Table

from mwutil import constants
from mwutil.local_config import MWUtilConfig
from mwutil.module import GlobalMWUtilModule


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
    autocomplete: list[str] | None = None
    confidential: bool = False
    allow_empty: bool = False

class Init(GlobalMWUtilModule):

    def get_description(self):
        return "Initialize a new MediaWiki development environment"

    def populate_subparser(self, parser: ArgumentParser, config: MWUtilConfig | None):
        parser.add_argument(
            "-a",
            "--advanced",
            action="store_true",
            help="Offer advanced configuration options",
        )

        parser.add_argument(
            "project_name",
            type=str,
            help="Name of the new MediaWiki project. Can contain only letters, numbers, hyphens and underscores.",
        )

    def execute(self, config: MWUtilConfig | None, args: Namespace):
        debug = os.getenv("MWUTIL_DEBUG")

        console = Console()

        if config is not None:
            print_failure(console, "You are already in a MediaWiki development environment.")
            return

        self.check_prerequisites(console, debug)
        self.validate_project_name(console, args.project_name)

        self.clone_mw_dev_kit(console, args.project_name, debug)
        os.chdir(args.project_name)

        self.configure_env(console, args.project_name, args.advanced, debug)
        # load env file instead of using the configured values so we can be sure we have all defaults
        dotenv.load_dotenv(os.path.join("config", ".env"))

        self.clone_core(console, debug)
        self.setup_config_files(console, debug)

        self.install(console, debug)
        self.post_install(console, debug)

        console.print(Panel.fit("[bold green]MediaWiki development environment initialized successfully![/bold green]", border_style="green"))

    @staticmethod
    def check_prerequisites(console: Console, debug: bool | None):
        with console.status("Checking prerequisites...", spinner="dots"):
            # check if "docker ps" returns valid output without sudo
            try:
                subprocess.run(
                    ["docker", "ps"],
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                    check=True,
                )
            except (subprocess.CalledProcessError, FileNotFoundError):
                print_failure(console, "Docker does not seem to be running or is not accessible.")
                print_failure(console, "Please ensure Docker is installed and running, and that you have permission to run it without sudo.")
                print_failure(console, "See https://askubuntu.com/a/477554 for guidance on managing Docker as a non-root user.")
                if debug:
                    console.print_exception()
                exit(1)

            # check if git review is available
            try:
                subprocess.run(
                    ["git", "review", "--version"],
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                    check=True,
                )
            except subprocess.CalledProcessError:
                print_failure(console, "git-review is not installed.")
                print_failure(console, "Please install git-review to proceed. On Fedora, you can install it via 'sudo dnf install git-review'.")
                if debug:
                    console.print_exception()
                exit(1)
        print_success(console, "All prerequisites met!")

    @staticmethod
    def validate_project_name(console: Console, name: str):
        project_name_pattern = re.compile(r"^[a-zA-Z0-9_-]+$")
        if not project_name_pattern.match(name):
            print_failure(
                console,
                "Invalid project name. It can contain only letters, numbers, hyphens and underscores."
            )
            exit(1)

        # check if folder exists
        if os.path.exists(name):
            print_failure(console, f"The folder '{name}' already exists in the current directory.")
            exit(1)

    @staticmethod
    def clone_mw_dev_kit(console: Console, project_name: str, debug: bool | None):
        method = questionary.select(
            "Choose a method to clone the Dev Kit",
            choices=[
                "Clone via HTTPS",
                "Clone via SSH",
                "Custom Origin"
            ],
        ).ask()
        if method == "Clone via HTTPS":
            repo_url = "https://github.com/SomeMWDev/mw-dev-kit.git"
        elif method == "Clone via SSH":
            repo_url = "git@github.com:SomeMWDev/mw-dev-kit.git"
        else:
            repo_url = questionary.text("Enter the custom repository URL:").ask()
        with console.status("Cloning mw-dev-kit...", spinner="dots"):
            run_command(console, ["git", "clone", repo_url, project_name], debug)
        print_success(console, f"Successfully cloned mw-dev-kit into '{project_name}'.")

    @staticmethod
    def get_git_option(option: str) -> str | None:
        try:
            result = subprocess.run(
                ["git", "config", "--get", option],
                stdout=subprocess.PIPE,
                stderr=subprocess.PIPE,
                check=True,
                text=True,
            )
            return result.stdout.strip()
        except subprocess.CalledProcessError:
            return ""

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
            autocomplete=constants.SUPPORTED_BRANCHES,
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
            default=lambda options, project_name: Init.get_git_option("user.name"),
            examples=["YourUsername"],
        ),
        EnvOption(
            "GIT_EMAIL",
            "Enter your Git email you generally use",
            validation_pattern=r"^[^@]+@[^@]+\.[^@]+$",
            default=lambda options, project_name: Init.get_git_option("user.email"),
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

    @staticmethod
    def configure_env(console: Console, project_name: str, advanced: bool, debug: bool | None):
        with console.status("Copying example environment file...", spinner="dots"):
            run_command(console, ["cp", "config/.env.example", "config/.env"], debug)
        print_success(console, "Copied .env.example to .env.")

        console.clear()

        options = {}
        for option in Init.ENV_OPTIONS:
            if (option.mode == EnvOptionMode.ADVANCED and not advanced) or option.mode == EnvOptionMode.AUTOMATIC:
                default = option.default
                if default is None:
                    raise ValueError(f"No default value for non-interactive option {option.key}")
                elif callable(default):
                    value = default(options, project_name)
                    if option.validation_pattern:
                        if not re.match(option.validation_pattern, value):
                            print_warning(console, f"Auto-configured value for {option.key} does not match validation pattern!")
                    options[option.key] = value
                else:
                    options[option.key] = default
                continue

            default_value = ""
            if callable(option.default):
                default_value = option.default(options, project_name)
            elif option.default is not None:
                default_value = option.default

            table = Table.grid(padding=(0, 2))
            table.add_column("Field", style="bold cyan", no_wrap=True)
            table.add_column("Value", style="white")

            table.add_row("Key", option.key)
            if str(default_value) != "":
                table.add_row("Default", str(default_value))
            if option.examples:
                table.add_row("Examples", ", ".join(option.examples))
            if option.reference:
                table.add_row("Reference", f"[link={option.reference}]{option.reference}[/link]")
            if option.validation_pattern:
                table.add_row("Validation", option.validation_pattern)
            table.add_row("Can be empty", "Yes" if option.allow_empty else "No")
            if option.mode == EnvOptionMode.ADVANCED:
                table.add_row("Mode", "Advanced")

            console.print(Panel.fit(table, title=f"Configure: [bold green]{option.prompt}[/bold green]", border_style="green"))

            while True:
                message = f"Enter value for {option.key}"
                if str(default_value) != "":
                    message += " (press Enter for default)"
                message += ":"
                if option.confidential:
                    if option.autocomplete:
                        print_warning(console, "Autocomplete is not supported for confidential inputs.")
                    answer = questionary.password(
                        message,
                        default=default_value
                    ).ask()
                else:
                    if option.autocomplete:
                        answer = questionary.autocomplete(
                            message,
                            choices=option.autocomplete,
                            default=default_value
                        ).ask()
                    else:
                        answer = questionary.text(
                            message,
                            default=default_value
                        ).ask()

                if option.validation_pattern and not re.match(option.validation_pattern, answer):
                    print_failure(console, f"Invalid input. Must match: {option.validation_pattern}")
                    continue

                options[option.key] = answer
                console.clear()
                break

        console.clear()
        # overwrite .env with the collected options. Assume keys already exist in .env.example
        with console.status("Configuring .env file...", spinner="dots"):
            for key, value in options.items():
                env_file = os.path.join("config", ".env")
                dotenv.set_key(env_file, key, value)

        print_success(console, "Successfully configured .env file.")

    @staticmethod
    def clone_core(console: Console, debug: bool | None):
        with console.status("Cloning MediaWiki core (this may take some time)...", spinner="dots"):
            command = [
                "git",
                "clone",
                f"ssh://{os.getenv("GERRIT_USERNAME")}@gerrit.wikimedia.org:29418/mediawiki/core",
                "-b",
                os.getenv("MW_BRANCH"),
            ]
            run_command(console, command, debug)
        print_success(console, "Successfully cloned MediaWiki core.")

    @staticmethod
    def setup_config_files(console: Console, debug: bool | None):
        with console.status("Linking composer.local.json...", spinner="dots"):
            run_command(console, [
                "ln",
                "core-composer.local.json",
                "core/composer.local.json",
            ], debug)
        print_success(console, "Linked composer.local.json.")

        with console.status("Creating .mwutil.json...", spinner="dots"):
            default_config = {}
            with open(".mwutil.json", "w") as f:
                import json
                json.dump(default_config, f, indent=4)
        print_success(console, "Created .mwutil.json.")

        with console.status("Copying and linking default LocalSettings.php...", spinner="dots"):
            run_command(console, [
                "cp",
                "LocalSettings.default.php",
                "LocalSettings.php",
            ], debug)
            run_command(console, [
                "ln",
                "LocalSettings.php",
                "core/LocalSettings.php",
            ], debug)
        print_success(console, "Copied and linked LocalSettings.php.")

    @staticmethod
    def install(console: Console, debug: bool | None):
        with console.status("Starting the containers...", spinner="dots"):
            run_command(console, ["mwutil", "up"], debug)
        print_success(console, "Containers started.")

        with console.status("Installing composer dependencies...", spinner="dots"):
            run_command(console, ["mwutil", "bash", "composer", "install"], debug)
        print_success(console, "Composer dependencies installed.")

        with console.status("Installing MediaWiki...", spinner="dots"):
            run_command(console, ["mwutil", "reset"], debug)
        print_success(console, "MediaWiki installed successfully!")

    @staticmethod
    def post_install(console: Console, debug: bool | None):
        with console.status("Setting up git-review in the local core git repository...", spinner="dots"):
            run_command(console, ["mwutil", "setup-gerrit"], debug, "core")
        print_success(console, "git-review setup completed.")

        clone_vector = questionary.confirm(
            "Do you want to clone the Vector skin?",
            default=True,
        ).ask()
        if clone_vector:
            with console.status("Cloning Vector skin into skins/ folder...", spinner="dots"):
                run_command(console, ["mwutil", "clone", "skin", "gerrit", "Vector"], debug)
            print_success(console, "Cloned Vector skin.")

            with console.status("Enabling Vector as the default skin in LocalSettings.php...", spinner="dots"):
                line = r"MediaWikiConfig::getInstance()->Vector( true );"
                with open("LocalSettings.php", "a") as f:
                    f.write(f"\n{line}\n")
            print_success(console, "Enabled Vector as the default skin.")

def print_success(console: Console, message: str):
    console.print(f"[bold green]✔ {message}")

def print_failure(console: Console, message: str):
    console.print(f"[bold red]✖ {message}")

def print_warning(console: Console, message: str):
    console.print(f"[bold yellow]⚠ {message}")

def print_normal(console: Console, message: str):
    console.print(f"[white]{message}")

def print_info(console: Console, message: str):
    console.print(f"[bold blue]ℹ {message}")

def print_detail(console: Console, message: str):
    console.print(f"[bright black]{message}")

def run_command(console: Console, command: list[str], debug: bool | None, cwd: str | None = None):
    # if debug is enabled, print the command being run
    if debug:
        print_detail(console, f"Running command: {' '.join(command)}")
    try:
        # if debug is enabled, print output to console. otherwise, suppress output
        stdout, stderr = (None, None) if debug else (subprocess.DEVNULL, subprocess.DEVNULL)
        subprocess.run(
            command,
            check=True,
            stdout=stdout,
            stderr=stderr,
            cwd=cwd
        )
    except subprocess.CalledProcessError:
        if debug:
            console.print_exception()
        raise
