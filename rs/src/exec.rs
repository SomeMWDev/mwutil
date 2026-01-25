use crate::config::MWUtilConfig;
use crate::types::Container;
use anyhow::{anyhow, Context};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::thread::JoinHandle;

pub trait ContainerSupport {
    fn in_container(
        &self,
        config: &MWUtilConfig,
        container: Container,
        exec_options: Option<&[String]>
    ) -> anyhow::Result<Self> where Self: Sized;
}

impl ContainerSupport for Command {

    fn in_container(
        &self,
        config: &MWUtilConfig,
        container: Container,
        exec_options: Option<&[String]>
    ) -> anyhow::Result<Self> {
        let mut docker_cmd = create_docker_compose_command(&config.base_dir);
        let mut cmd_args: Vec<String> = vec![
            "exec".into(),
        ];
        if let Some(workdir) = self.get_current_dir() {
            cmd_args.push("-w".into());
            let workdir_str = workdir.to_string_lossy();
            if workdir.is_relative() {
                cmd_args.push(config.mw_install_path.clone() + "/" + workdir_str.as_ref())
            } else {
                cmd_args.push(workdir_str.to_string());
            }
        }
        cmd_args.extend_from_slice(exec_options.unwrap_or_default());
        cmd_args.extend([
            container.to_string(),
            self.get_program()
                .to_string_lossy()
                .to_string()
        ]);
        cmd_args.extend(self.get_args().map(|s| s.to_string_lossy().into_owned()));

        docker_cmd.args(&cmd_args);
        Ok(docker_cmd)
    }

}

pub trait CommandExt {
    fn live_output(&mut self) -> anyhow::Result<(ExitStatus, String, String)>;
}

impl CommandExt for Command {
    fn live_output(&mut self) -> anyhow::Result<(ExitStatus, String, String)> {
        self.stdout(Stdio::piped());
        self.stderr(Stdio::piped());

        let mut child = self.spawn()
            .context("Failed to spawn child")?;

        let stdout = child.stdout.take()
            .ok_or_else(|| anyhow!("Failed to take stdout"))?;
        let stderr = child.stderr.take()
            .ok_or_else(|| anyhow!("Failed to take stderr"))?;

        let stdout_handle: JoinHandle<anyhow::Result<String>> = thread::spawn(move || {
            let reader = BufReader::new(stdout);
            let mut collected = String::new();
            for line in reader.lines() {
                let line = line?;
                println!("{}", line);
                collected.push_str(&line);
                collected.push('\n');
            }
            Ok(collected)
        });

        let stderr_handle: JoinHandle<anyhow::Result<String>> = thread::spawn(move || {
            let reader = BufReader::new(stderr);
            let mut collected = String::new();
            for line in reader.lines() {
                let line = line?;
                eprintln!("{}", line);
                collected.push_str(&line);
                collected.push('\n');
            }
            Ok(collected)
        });

        let status = child.wait()
            .context("Failed to wait for child")?;
        let stdout_output = stdout_handle.join()
            .map_err(|e| anyhow!("Failed to handle stdout: {:?}", e))??;
        let stderr_output = stderr_handle.join()
            .map_err(|e| anyhow!("Failed to handle stderr: {:?}", e))??;

        Ok((status, stdout_output, stderr_output))
    }
}

pub fn create_docker_compose_command(base_dir: &Path) -> Command {
    let mut cmd = Command::new("docker");
    cmd.args(["compose", "--env-file", "config/.env", "--progress", "plain"]);
    cmd.current_dir(base_dir);
    cmd
}

pub enum DbCommandUser {
    Mw,
    Root,
}

pub enum DbCommandType {
    Dump,
    Query,
}

pub enum DbCommandDatabase {
    None,
    Mw,
}

pub fn create_db_command(
    config: &MWUtilConfig,
    cmd_type: DbCommandType,
    user: DbCommandUser,
    args: Option<&[&str]>,
    exec_options: Option<&[String]>,
    database: Option<DbCommandDatabase>,
) -> anyhow::Result<Command> {
    let mut cmd = Command::new(match cmd_type {
        DbCommandType::Dump => config.db_type.get_dump_command(),
        DbCommandType::Query => config.db_type.get_query_command(),
    });

    match database.unwrap_or(DbCommandDatabase::Mw) {
        DbCommandDatabase::None => None,
        DbCommandDatabase::Mw => config.mw_database.as_deref(),
    }.map(|db|cmd.arg(db));

    match user {
        DbCommandUser::Mw => {
            cmd.args([
                &format!("-u{}", config.db_user.clone().ok_or_else(|| anyhow!("DB user not set!"))?),
                &format!("-p{}", config.db_password.clone().ok_or_else(|| anyhow!("DB password not set!"))?),
            ]);
        },
        DbCommandUser::Root => {
            let root_password = config.db_root_password
                .as_ref()
                .ok_or_else(|| anyhow!("DB root password not set!"))?;
            cmd.args(["-uroot", &format!("-p{root_password}")]);
        },
    }

    if let Some(args) = args {
        cmd.args(args);
    }

    cmd.in_container(config, Container::Database(config.db_type.clone()), exec_options)
}

pub fn run_sql_query(
    config: &MWUtilConfig,
    user: DbCommandUser,
    database: Option<DbCommandDatabase>,
    query: &str,
) -> anyhow::Result<ExitStatus> {
    create_db_command(config, DbCommandType::Query, user, Some(&["-e", query]), None, database)?
        .status()
        .map_err(|x|anyhow!(x))
}
