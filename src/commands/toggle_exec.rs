use clap::Args;
use hyprland::data::Workspaces;
use hyprland::dispatch::{Dispatch, DispatchType};
use hyprland::prelude::*;
use itertools::Itertools;
use std::thread;
use std::time::Duration;

/// Toggles an special workspace, and runs exec for the workspace if it isn't created
#[derive(Args)]
pub struct ToggleExecCommand {
    /// Name for the special workspace (without `special:`)
    #[arg(short, long)]
    name: String,

    /// The command to exec
    #[arg(short, long)]
    exec: String,

    /// The amount of retries for trying to toggle the special workspace, runs with a 100ms interval
    #[arg(long, default_value_t = 30)]
    max_retries: u32,

    /// The size of the window/workspace (in percent of screen)
    #[arg(short, long, default_value = "80x80")]
    size: String,

    /// The position in screen of the window/workspace
    #[arg(short, long, default_value_t = true)]
    center: bool,

    /// The type of the window/workspace, floating or tiled
    #[arg(short, long, default_value = "float")]
    wintype: String,

    /// Whether the workspace should be hidden at startup
    #[arg(long, default_value_t = false)]
    hidden: bool,
}

const TIMEOUT: Duration = Duration::from_millis(100);

impl ToggleExecCommand {
    pub fn run(self) -> anyhow::Result<()> {
        let Self {
            name,
            exec,
            max_retries,
            size,
            center,
            wintype,
            hidden,
        } = self;

        let target_workspace_name = format!("special:{name}");

        let size = size
            .split('x')
            .map(|x| x.parse::<u32>().unwrap())
            .collect_vec();

        let size_ = "size ".to_owned() + &size[0].to_string() + "% " + &size[1].to_string() + "%;";
        let center_ = if center { "center;".to_owned() } else { "".to_owned() };
        let wintype_ = "".to_owned() + &wintype + ";";

        let all_val = wintype_ + &size_ + &center_;
        print!("{}", all_val);

        let workspaces = Workspaces::get()?.collect_vec();
        let workspace_is_spawned = workspaces
            .iter()
            .any(|workspace| workspace.name == target_workspace_name);

        if !workspace_is_spawned {
            let exec_command = format!("[workspace {target_workspace_name} {all_val};noanim] {exec}");

            Dispatch::call(DispatchType::Exec(&exec_command))?;

            for _ in 0..max_retries {
                thread::sleep(TIMEOUT);

                let workspaces = Workspaces::get()?.collect_vec();
                let is_spawned = workspaces
                    .iter()
                    .any(|workspace| workspace.name == target_workspace_name);

                if is_spawned {
                    break;
                }
            }
        }

        Dispatch::call(DispatchType::ToggleSpecialWorkspace(Some(name)))?;

        Ok(())
    }
}
