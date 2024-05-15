use crate::config::{Action, AvailableActions, AvailableLeaderKeys, Command, Config};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct UnresolvedConfig {
    workspaces: u8,
    border_width: Option<u16>,
    border_color: Option<u32>,
    leader: UnresolvedLeader,
    actions: Vec<UnresolvedActionEntry>,
    commands: Vec<UnresolvedCommandEntry>,
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
    Workspace1,
    Workspace2,
    Workspace3,
    Workspace4,
    Workspace5,
    Workspace6,
    Workspace7,
    Workspace8,
    Workspace9,
    Workspace0,
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
    type Error = ();

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

        if value.workspaces.gt(&10) || value.workspaces.eq(&0) {
            return Err(());
        }

        Ok(Self {
            workspaces: value.workspaces,
            border_color: value.border_color.unwrap_or(0xff0000),
            border_width: value.border_width.unwrap_or(1),
            actions,
            leader,
            commands,
        })
    }
}

impl TryFrom<UnresolvedActionEntry> for Action {
    type Error = ();

    fn try_from(value: UnresolvedActionEntry) -> Result<Self, Self::Error> {
        Ok(Self {
            action: value.action.into(),
            key: value.key.as_str().try_into()?,
            modifier: value
                .modifiers
                .into_iter()
                .fold(0, |acc, modifier| acc + u32::from(modifier)),
        })
    }
}

impl TryFrom<UnresolvedCommandEntry> for Command {
    type Error = ();

    fn try_from(value: UnresolvedCommandEntry) -> Result<Self, Self::Error> {
        let a = Self {
            command: value.command,
            key: value.key.as_str().try_into()?,
            modifier: value
                .modifiers
                .into_iter()
                .fold(0, |acc, modifier| acc + u32::from(modifier)),
        };
        Ok(a)
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
            UnresolvedAction::Close => AvailableActions::Close,
            UnresolvedAction::Reload => AvailableActions::Reload,
            UnresolvedAction::Workspace1 => AvailableActions::Workspace1,
            UnresolvedAction::Workspace2 => AvailableActions::Workspace2,
            UnresolvedAction::Workspace3 => AvailableActions::Workspace3,
            UnresolvedAction::Workspace4 => AvailableActions::Workspace4,
            UnresolvedAction::Workspace5 => AvailableActions::Workspace5,
            UnresolvedAction::Workspace6 => AvailableActions::Workspace6,
            UnresolvedAction::Workspace7 => AvailableActions::Workspace7,
            UnresolvedAction::Workspace8 => AvailableActions::Workspace8,
            UnresolvedAction::Workspace9 => AvailableActions::Workspace9,
            UnresolvedAction::Workspace0 => AvailableActions::Workspace0,
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
