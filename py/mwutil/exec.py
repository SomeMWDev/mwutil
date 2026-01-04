import os
import shlex
import subprocess
from pathlib import Path
from subprocess import CompletedProcess

from mwutil.local_config import MWUtilConfig
from mwutil.data_state import get_profiles


def run_docker_command(config: MWUtilConfig, command: list[str], capture_output=False, input_text: str | None = None, text: bool = True) -> CompletedProcess:
    cmd = [
        "docker", "compose", "-p", os.getenv("DOCKER_COMPOSE_PROJECT_NAME")
    ]

    for name in get_profiles(config):
        cmd += ["--profile", name]

    cmd += command

    return subprocess.run(cmd, cwd=config.basedir, capture_output=capture_output, input=input_text, text=text)

def run_container_command(
    config: MWUtilConfig,
    command: list[str],
    container_name: str = "mediawiki",
    exec_options: list[str] | None = None,
    capture_output=False,
    input_text: str | None = None,
    text: bool = True,
    workdir: str | None = None
) -> CompletedProcess:
    if command[0] != "bash":
        # avoid "OCI runtime exec failed: exec failed: unable to start container process: ..."
        command = ["bash", "-c"] + [shlex.join(command)]
    if workdir and not workdir.startswith("/"):
        # resolve relative paths against the MW installation directory
        workdir = config.mw_install_path + "/" + workdir
    # For mediawiki containers, use the MW installation directory provided in .env by default (#13)
    workdir_option = ["-w", (workdir or config.mw_install_path)] if container_name.startswith("mediawiki") else []
    exec_options = workdir_option + (exec_options or [])
    return run_docker_command(
        config,
        ["exec"] + exec_options + [container_name] + command,
        capture_output=capture_output,
        input_text=input_text,
        text=text
    )

def run_command(command: list[str], path: Path | None = None, capture_output=False) -> CompletedProcess:
    return subprocess.run(command, cwd=path, capture_output=capture_output)

def run_wiki_db_command(
    config: MWUtilConfig,
    command: list[str] | str,
    options: list[str],
    **kwargs
) -> CompletedProcess:
    # TODO move to config
    user = os.getenv("MWC_DB_USER")
    password = os.getenv("MWC_DB_PASSWORD")
    return run_db_command(
        config,
        user,
        password,
        command,
        options,
        **kwargs
    )

def run_root_db_command(
    config: MWUtilConfig,
    command: list[str] | str,
    options: list[str],
    **kwargs
) -> CompletedProcess:
    user = "root"
    # TODO move to config
    password = os.getenv("MWC_DB_ROOT_PASSWORD")
    return run_db_command(
        config,
        user,
        password,
        command,
        options,
        **kwargs
    )

def run_db_command(
        config: MWUtilConfig,
        user: str,
        password: str,
        command: list[str] | str,
        options: list[str],
        exec_options: list[str] | None = None,
        capture_output=False,
        input_text: str | None = None,
        text: bool = True
) -> CompletedProcess:
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
    return run_wiki_db_command(
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
        return None
