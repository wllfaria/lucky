use std::ops::Add;

use crate::{
    color_parser::Color,
    config::{
        Action, ActionModifier, AutoCommand, AvailableActions, AvailableLeaderKeys, Command, Config,
    },
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct UnresolvedConfig {
    workspaces: u8,
    border_width: Option<u16>,
    border_color: Option<String>,
    focus_follow_mouse: Option<bool>,
    active_border_color: Option<String>,
    focus_new_clients: Option<bool>,
    leader: UnresolvedLeader,
    actions: Vec<UnresolvedActionEntry>,
    commands: Vec<UnresolvedCommandEntry>,
    startup_commands: Option<Vec<String>>,
}

#[derive(Deserialize)]
enum UnresolvedLeader {
    Control,
    Shift,
    Mod1,
}

#[derive(Deserialize, Clone)]
enum UnresolvedModifier {
    Leader,
    Control,
    Shift,
    Mod1,
}

#[derive(Deserialize)]
struct UnresolvedActionEntry {
    modifiers: Vec<UnresolvedModifier>,
    key: String,
    action: UnresolvedAction,
}

#[derive(Deserialize)]
struct UnresolvedCommandEntry {
    modifiers: Vec<UnresolvedModifier>,
    key: String,
    command: String,
}

#[derive(Deserialize)]
enum UnresolvedAction {
    FocusLeft,
    FocusDown,
    FocusUp,
    FocusRight,
    MoveLeft,
    MoveDown,
    MoveUp,
    MoveRight,
    Close,
    Reload,
    Quit,
    Workspace1,
    Workspace2,
    Workspace3,
    Workspace4,
    Workspace5,
    Workspace6,
    Workspace7,
    Workspace8,
    Workspace9,
    MoveToWorkspace1,
    MoveToWorkspace2,
    MoveToWorkspace3,
    MoveToWorkspace4,
    MoveToWorkspace5,
    MoveToWorkspace6,
    MoveToWorkspace7,
    MoveToWorkspace8,
    MoveToWorkspace9,
    Fullscreen,
}

pub enum ConfigError {
    Key(String),
    Workspaces(String),
    BorderWidth(String),
    BorderColor(String),
    InvalidCommand(String),
    Color(String),
}

impl From<AvailableLeaderKeys> for UnresolvedModifier {
    fn from(value: AvailableLeaderKeys) -> Self {
        match value {
            AvailableLeaderKeys::Mod1 => UnresolvedModifier::Mod1,
            AvailableLeaderKeys::Shift => UnresolvedModifier::Shift,
            AvailableLeaderKeys::Control => UnresolvedModifier::Control,
        }
    }
}

impl TryFrom<UnresolvedConfig> for Config {
    type Error = ConfigError;

    fn try_from(value: UnresolvedConfig) -> Result<Self, Self::Error> {
        let mut value = value;

        let leader = match value.leader {
            UnresolvedLeader::Shift => AvailableLeaderKeys::Shift,
            UnresolvedLeader::Mod1 => AvailableLeaderKeys::Mod1,
            UnresolvedLeader::Control => AvailableLeaderKeys::Control,
        };

        value.actions.iter_mut().for_each(|action| {
            action.modifiers.iter_mut().for_each(|modifier| {
                if let UnresolvedModifier::Leader = modifier {
                    *modifier = leader.clone().into();
                }
            })
        });
        value.commands.iter_mut().for_each(|command| {
            command.modifiers.iter_mut().for_each(|modifier| {
                if let UnresolvedModifier::Leader = modifier {
                    *modifier = leader.clone().into();
                }
            })
        });

        let mut actions: Vec<Action> = vec![];
        for action in value.actions.into_iter() {
            actions.push(action.try_into()?);
        }

        let mut commands: Vec<Command> = vec![];
        for command in value.commands.into_iter() {
            commands.push(command.try_into()?);
        }

        let mut startup_commands: Vec<AutoCommand> = vec![];
        for auto_command in value.startup_commands.unwrap_or_default().into_iter() {
            startup_commands.push(auto_command.try_into()?);
        }

        if value.workspaces.gt(&9) || value.workspaces.eq(&0) {
            return Err(ConfigError::Workspaces(format!(
                "workspaces = {}: number of workspaces must be greater than 0, and up to 9",
                value.workspaces
            )));
        }

        let border_color = Color::try_from(value.border_color.unwrap_or_default())
            .map_err(|e| ConfigError::BorderColor(e.to_string()))?
            .0;

        let active_border_color = Color::try_from(value.active_border_color.unwrap_or_default())
            .map_err(|e| ConfigError::BorderColor(e.to_string()))?
            .0;

        Ok(Config {
            workspaces: value.workspaces,
            border_width: value.border_width.unwrap_or(1),
            border_color,
            active_border_color,
            focus_follow_mouse: value.focus_follow_mouse.unwrap_or(true),
            focus_new_clients: value.focus_new_clients.unwrap_or(true),
            actions,
            leader,
            commands,
            startup_commands,
        })
    }
}

impl TryFrom<UnresolvedActionEntry> for Action {
    type Error = ConfigError;

    fn try_from(value: UnresolvedActionEntry) -> Result<Self, Self::Error> {
        Ok(Action {
            action: value.action.into(),
            key: value.key.as_str().try_into()?,
            modifier: ActionModifier::new(
                value
                    .modifiers
                    .into_iter()
                    .fold(0, |acc, modifier| acc.add(u32::from(modifier))),
            ),
        })
    }
}

impl TryFrom<UnresolvedCommandEntry> for Command {
    type Error = ConfigError;

    fn try_from(value: UnresolvedCommandEntry) -> Result<Self, Self::Error> {
        let (command, args): (String, Args) = value
            .command
            .split_once(' ')
            .map(|(left, right)| (left.to_string(), right.into()))
            .unwrap_or((value.command, Args(vec![])));

        Ok(Command {
            command: command.to_string(),
            key: value.key.as_str().try_into()?,
            modifier: value
                .modifiers
                .into_iter()
                .fold(0, |acc, modifier| acc + u32::from(modifier)),
            args: args.0,
        })
    }
}

impl TryFrom<String> for AutoCommand {
    type Error = ConfigError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Err(ConfigError::InvalidCommand(
                "statup_command cannot be empty".to_string(),
            ));
        }

        let (command, args): (String, Args) = value
            .trim_end_matches('&')
            .trim()
            .split_once(' ')
            .map(|(left, right)| (left.to_string(), right.into()))
            .unwrap_or((value, Args(vec![])));

        Ok(AutoCommand {
            command,
            args: args.0,
        })
    }
}

impl From<UnresolvedAction> for AvailableActions {
    fn from(value: UnresolvedAction) -> Self {
        match value {
            UnresolvedAction::FocusLeft => AvailableActions::FocusLeft,
            UnresolvedAction::FocusDown => AvailableActions::FocusDown,
            UnresolvedAction::FocusUp => AvailableActions::FocusUp,
            UnresolvedAction::FocusRight => AvailableActions::FocusRight,
            UnresolvedAction::MoveLeft => AvailableActions::MoveLeft,
            UnresolvedAction::MoveDown => AvailableActions::MoveDown,
            UnresolvedAction::MoveUp => AvailableActions::MoveUp,
            UnresolvedAction::MoveRight => AvailableActions::MoveRight,
            UnresolvedAction::Quit => AvailableActions::Quit,
            UnresolvedAction::Close => AvailableActions::Close,
            UnresolvedAction::Reload => AvailableActions::Reload,
            UnresolvedAction::Fullscreen => AvailableActions::Fullscreen,
            UnresolvedAction::Workspace1 => AvailableActions::Workspace1,
            UnresolvedAction::Workspace2 => AvailableActions::Workspace2,
            UnresolvedAction::Workspace3 => AvailableActions::Workspace3,
            UnresolvedAction::Workspace4 => AvailableActions::Workspace4,
            UnresolvedAction::Workspace5 => AvailableActions::Workspace5,
            UnresolvedAction::Workspace6 => AvailableActions::Workspace6,
            UnresolvedAction::Workspace7 => AvailableActions::Workspace7,
            UnresolvedAction::Workspace8 => AvailableActions::Workspace8,
            UnresolvedAction::Workspace9 => AvailableActions::Workspace9,
            UnresolvedAction::MoveToWorkspace1 => AvailableActions::MoveToWorkspace1,
            UnresolvedAction::MoveToWorkspace2 => AvailableActions::MoveToWorkspace2,
            UnresolvedAction::MoveToWorkspace3 => AvailableActions::MoveToWorkspace3,
            UnresolvedAction::MoveToWorkspace4 => AvailableActions::MoveToWorkspace4,
            UnresolvedAction::MoveToWorkspace5 => AvailableActions::MoveToWorkspace5,
            UnresolvedAction::MoveToWorkspace6 => AvailableActions::MoveToWorkspace6,
            UnresolvedAction::MoveToWorkspace7 => AvailableActions::MoveToWorkspace7,
            UnresolvedAction::MoveToWorkspace8 => AvailableActions::MoveToWorkspace8,
            UnresolvedAction::MoveToWorkspace9 => AvailableActions::MoveToWorkspace9,
        }
    }
}

impl From<UnresolvedModifier> for u32 {
    fn from(value: UnresolvedModifier) -> u32 {
        match value {
            UnresolvedModifier::Shift => 0x00000001,
            UnresolvedModifier::Control => 0x00000004,
            UnresolvedModifier::Mod1 => 0x00000008,
            _ => 0x00000000,
        }
    }
}

pub struct Args(Vec<String>);

impl From<&str> for Args {
    fn from(value: &str) -> Self {
        let args = value
            .split_whitespace()
            .map(|slice| slice.to_string())
            .collect::<Vec<String>>();

        Self(args)
    }
}
