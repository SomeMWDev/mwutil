use std::process::Command;

pub trait ContainerSupport {
    fn in_container(self, container: &str, exec_options: Option<Vec<String>>) -> Self;
}

impl ContainerSupport for Command {

    fn in_container(
        self,
        container: &str,
        exec_options: Option<Vec<String>>
    ) -> Self {
        let mut docker_cmd = Command::new("docker");
        let mut cmd_args: Vec<String> = vec![
            "compose".into(),
            "--env-file".into(),
            "config/.env".into(),
            "exec".into(),
        ];
        if let Some(workdir) = self.get_current_dir() {
            cmd_args.push("-w".into());
            // TODO fix unwrap
            cmd_args.push(workdir.to_str().unwrap().to_string());
        }
        cmd_args.extend(exec_options.unwrap_or_default());
        cmd_args.extend([
            container.into(),
            self.get_program().to_string_lossy().into_owned()
        ]);
        cmd_args.extend(self.get_args().map(|s| s.to_string_lossy().into_owned()));

        docker_cmd.args(&cmd_args);
        docker_cmd
    }

}
