import os
import re
import subprocess
from argparse import ArgumentParser, Namespace

import dotenv
import questionary
from questionary import Separator
from rich.console import Console
from rich.panel import Panel
from rich.progress import Progress
from rich.prompt import Prompt, Confirm
from rich.table import Table

from mwutil.env import EnvOptionMode, ENV_OPTIONS, EnvOption
from mwutil.local_config import MWUtilConfig
from mwutil.module import GlobalMWUtilModule


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

    def execute(self, config: MWUtilConfig | None, args: Namespace):
        debug = os.getenv("MWUTIL_DEBUG")

        console = Console()

        if config is not None:
            print_failure(console, "You are already in a MediaWiki development environment.")
            return

        self.check_prerequisites(console, debug)
        project_name = self.get_project_name(console)

        self.clone_mw_dev_kit(console, project_name, debug)
        os.chdir(project_name)

        self.configure_env(console, project_name, args.advanced, debug)
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
            except (subprocess.CalledProcessError, FileNotFoundError):
                print_failure(console, "git-review is not installed.")
                print_failure(console, "Please install git-review to proceed. On Fedora, you can install it via 'sudo dnf install git-review'.")
                if debug:
                    console.print_exception()
                exit(1)
        print_success(console, "All prerequisites met!")

    @staticmethod
    def get_project_name(console: Console) -> str:
        project_name_pattern = re.compile(r"^[a-zA-Z0-9_-]+$")

        print_info(console, "The project name will be used as the folder name for your new MediaWiki development environment.")
        print_info(console, "It can contain only letters, numbers, hyphens and underscores.")
        while True:
            name = Prompt.ask("Enter a name for your new MediaWiki project")
            if project_name_pattern.match(name):
                # check if folder exists
                if os.path.exists(name):
                    print_failure(console, f"The folder '{name}' already exists in the current directory.")
                else:
                    return name
            else:
                print_failure(
                    console,
                    "Invalid project name. It can contain only letters, numbers, hyphens and underscores."
                )

    @staticmethod
    def clone_mw_dev_kit(console: Console, project_name: str, debug: bool | None):
        # TODO replace questionary here with rich prompts
        method = questionary.select(
            "Choose a method to clone the Dev Kit",
            choices=[
                "Clone via HTTPS",
                "Clone via SSH",
                Separator(),
                "Custom Origin"
            ],
        ).ask()
        if method == "Clone via HTTPS":
            repo_url = "https://github.com/SomeMWDev/mw-dev-kit.git"
        elif method == "Clone via SSH":
            repo_url = "git@github.com:SomeMWDev/mw-dev-kit.git"
        else:
            repo_url = Prompt.ask("Enter the custom repository URL")
        if repo_url is None or repo_url.strip() == "":
            print_failure(console, "No repository URL provided. Exiting.")
            exit(1)
        with console.status("Cloning mw-dev-kit...", spinner="dots"):
            run_command(console, ["git", "clone", repo_url, project_name], debug)
        print_success(console, f"Successfully cloned mw-dev-kit into '{project_name}'.")

    @staticmethod
    def configure_env(console: Console, project_name: str, advanced: bool, debug: bool | None):
        with console.status("Copying example environment file...", spinner="dots"):
            run_command(console, ["cp", "config/.env.example", "config/.env"], debug)
        print_success(console, "Copied .env.example to .env.")

        console.clear()

        options = {}

        def is_manual_question(opt: EnvOption):
            return not (
                opt.mode == EnvOptionMode.ADVANCED and not advanced
                or opt.mode == EnvOptionMode.AUTOMATIC
            )

        expected_question_amount = len(list(filter(is_manual_question, ENV_OPTIONS)))
        answered_questions = 0
        for option in ENV_OPTIONS:
            if not is_manual_question(option):
                default = option.default
                if default is None:
                    raise ValueError(f"No default value for non-interactive option {option.key}")
                elif callable(default):
                    value = default(options, project_name)
                    if option.validation_pattern:
                        if not re.fullmatch(option.validation_pattern, value):
                            print_warning(console, f"Auto-configured value for {option.key} does not match validation pattern!")
                        if not option.allow_empty and value == "":
                            print_warning(console, f"Auto-configured value for {option.key} is empty but empty values are not allowed!")
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

            with Progress() as progress:
                progress.add_task(
                    "[green]Progress:",
                    total=expected_question_amount,
                    completed=answered_questions,
                )
            console.print(Panel.fit(table, title=f"Configure: [bold green]{option.prompt}[/bold green]", border_style="green"))

            def validate(answer_value: str | None, is_first: bool = False) -> bool:
                if answer_value is None:
                    if not is_first:
                        print_failure(console, "Invalid input!")
                    return False
                if not option.allow_empty and answer_value == "":
                    print_failure(console, "This value cannot be empty.")
                    return False
                if option.validation_pattern and not re.fullmatch(option.validation_pattern, answer_value):
                    print_failure(console, f"Invalid format. Must match: {option.validation_pattern}")
                    return False
                return True

            answer = None
            first = True
            while not validate(answer, first):
                answer = Prompt.ask(
                    f"Enter value for {option.key}",
                    console=console,
                    default=default_value if not option.confidential else None,
                    password=option.confidential,
                )
                first = False
            answered_questions += 1
            options[option.key] = answer
            console.clear()

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

        clone_vector = Confirm.ask(
            "Do you want to clone the Vector skin?",
            default=True,
        )
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
