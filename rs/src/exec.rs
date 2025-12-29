use std::io::{BufRead, BufReader};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use anyhow::anyhow;
use crate::config::MWUtilConfig;

pub trait ContainerSupport {
    fn in_container(self, config: &MWUtilConfig, container: &str, exec_options: Option<&[String]>) -> Self;
}

impl ContainerSupport for Command {

    fn in_container(
        self,
        config: &MWUtilConfig,
        container: &str,
        exec_options: Option<&[String]>
    ) -> Self {
        let mut docker_cmd = create_docker_compose_command(config);
        let mut cmd_args: Vec<String> = vec![
            "exec".into(),
        ];
        if let Some(workdir) = self.get_current_dir() {
            cmd_args.push("-w".into());
            // TODO fix unwrap
            cmd_args.push(workdir.to_str().unwrap().to_string());
        }
        cmd_args.extend_from_slice(exec_options.unwrap_or_default());
        cmd_args.extend([
            container.into(),
            self.get_program().to_string_lossy().into_owned()
        ]);
        cmd_args.extend(self.get_args().map(|s| s.to_string_lossy().into_owned()));

        docker_cmd.args(&cmd_args);
        docker_cmd
    }

}

pub trait CommandExt {
    fn live_output(&mut self) -> std::io::Result<(ExitStatus, String, String)>;
}

impl CommandExt for Command {
    fn live_output(&mut self) -> std::io::Result<(ExitStatus, String, String)> {
        self.stdout(Stdio::piped());
        self.stderr(Stdio::piped());

        let mut child = self.spawn()?;

        let stdout = child.stdout.take().unwrap();
        let stderr = child.stderr.take().unwrap();

        let stdout_handle = thread::spawn(move || {
            let reader = BufReader::new(stdout);
            let mut collected = String::new();
            for line in reader.lines() {
                let line = line.unwrap();
                println!("{}", line);
                collected.push_str(&line);
                collected.push('\n');
            }
            collected
        });

        let stderr_handle = thread::spawn(move || {
            let reader = BufReader::new(stderr);
            let mut collected = String::new();
            for line in reader.lines() {
                let line = line.unwrap();
                eprintln!("{}", line);
                collected.push_str(&line);
                collected.push('\n');
            }
            collected
        });

        let status = child.wait()?;
        let stdout_output = stdout_handle.join().unwrap();
        let stderr_output = stderr_handle.join().unwrap();

        Ok((status, stdout_output, stderr_output))
    }
}

pub fn create_docker_compose_command(config: &MWUtilConfig) -> Command {
    let mut cmd = Command::new("docker");
    cmd.args(["compose", "--env-file", "config/.env"]);
    cmd.current_dir(config.base_dir.clone());
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

pub enum DbCommandDatabase<'a> {
    Custom(&'a str),
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
        DbCommandDatabase::Custom(name) => Some(name),
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

    Ok(cmd.in_container(config, config.db_type.get_container_name(), exec_options))
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
