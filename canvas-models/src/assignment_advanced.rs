use std::str::FromStr;

use thiserror::Error;

use crate::{DueDate, SectionId, UserId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GroupCategoryId(u64);

impl GroupCategoryId {
    pub fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum GroupCategoryIdParseError {
    #[error("group category id must be a positive integer")]
    Invalid,
}

impl FromStr for GroupCategoryId {
    type Err = GroupCategoryIdParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let trimmed = raw.trim();
        let value = trimmed
            .parse::<u64>()
            .map_err(|_| GroupCategoryIdParseError::Invalid)?;
        if value == 0 {
            return Err(GroupCategoryIdParseError::Invalid);
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerReviewMode {
    Automatic,
    Manual,
}

impl PeerReviewMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            PeerReviewMode::Automatic => "automatic",
            PeerReviewMode::Manual => "manual",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PeerReviewModeParseError {
    #[error("peer review mode must be 'automatic' or 'manual'")]
    Invalid,
}

impl FromStr for PeerReviewMode {
    type Err = PeerReviewModeParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "automatic" | "auto" => Ok(PeerReviewMode::Automatic),
            "manual" => Ok(PeerReviewMode::Manual),
            _ => Err(PeerReviewModeParseError::Invalid),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeerReviewSettings {
    mode: PeerReviewMode,
    assign_at: Option<DueDate>,
    due_at: Option<DueDate>,
}

impl PeerReviewSettings {
    pub fn new(
        mode: PeerReviewMode,
        assign_at: Option<DueDate>,
        due_at: Option<DueDate>,
    ) -> Result<Self, PeerReviewSettingsError> {
        match mode {
            PeerReviewMode::Automatic => {
                if assign_at.is_none() {
                    return Err(PeerReviewSettingsError::MissingAssignAt);
                }
            }
            PeerReviewMode::Manual => {
                if assign_at.is_some() {
                    return Err(PeerReviewSettingsError::ManualAssignAt);
                }
            }
        }
        if let (Some(assign_at), Some(due_at)) = (assign_at, due_at)
            && assign_at.inner() > due_at.inner()
        {
            return Err(PeerReviewSettingsError::AssignAfterDue);
        }
        Ok(Self {
            mode,
            assign_at,
            due_at,
        })
    }

    pub fn mode(self) -> PeerReviewMode {
        self.mode
    }

    pub fn assign_at(self) -> Option<DueDate> {
        self.assign_at
    }

    pub fn due_at(self) -> Option<DueDate> {
        self.due_at
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PeerReviewSettingsError {
    #[error("automatic peer reviews require an assign date")]
    MissingAssignAt,
    #[error("manual peer reviews cannot specify an assign date")]
    ManualAssignAt,
    #[error("peer review assign date must be on or before the due date")]
    AssignAfterDue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupAssignmentMode {
    Group,
    Individual,
}

impl GroupAssignmentMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            GroupAssignmentMode::Group => "group",
            GroupAssignmentMode::Individual => "individual",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum GroupAssignmentModeParseError {
    #[error("group assignment mode must be 'group' or 'individual'")]
    Invalid,
}

impl FromStr for GroupAssignmentMode {
    type Err = GroupAssignmentModeParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "group" | "grouped" => Ok(GroupAssignmentMode::Group),
            "individual" | "solo" => Ok(GroupAssignmentMode::Individual),
            _ => Err(GroupAssignmentModeParseError::Invalid),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GroupAssignmentSettings {
    Individual,
    Group { category_id: GroupCategoryId },
}

impl GroupAssignmentSettings {
    pub fn new(
        mode: GroupAssignmentMode,
        category_id: Option<GroupCategoryId>,
    ) -> Result<Self, GroupAssignmentSettingsError> {
        match mode {
            GroupAssignmentMode::Group => {
                let category_id = category_id
                    .ok_or(GroupAssignmentSettingsError::MissingCategory)?;
                Ok(Self::Group { category_id })
            }
            GroupAssignmentMode::Individual => {
                if category_id.is_some() {
                    return Err(GroupAssignmentSettingsError::CategoryNotAllowed);
                }
                Ok(Self::Individual)
            }
        }
    }

    pub fn mode(self) -> GroupAssignmentMode {
        match self {
            GroupAssignmentSettings::Individual => GroupAssignmentMode::Individual,
            GroupAssignmentSettings::Group { .. } => GroupAssignmentMode::Group,
        }
    }

    pub fn category_id(self) -> Option<GroupCategoryId> {
        match self {
            GroupAssignmentSettings::Individual => None,
            GroupAssignmentSettings::Group { category_id } => Some(category_id),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum GroupAssignmentSettingsError {
    #[error("group assignments require a group category id")]
    MissingCategory,
    #[error("individual assignments cannot specify a group category id")]
    CategoryNotAllowed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AssignmentOverrideDates {
    unlock_at: Option<DueDate>,
    due_at: Option<DueDate>,
    lock_at: Option<DueDate>,
}

impl AssignmentOverrideDates {
    pub fn new(
        unlock_at: Option<DueDate>,
        due_at: Option<DueDate>,
        lock_at: Option<DueDate>,
    ) -> Result<Self, AssignmentOverrideDatesError> {
        if let (Some(unlock_at), Some(due_at)) = (unlock_at, due_at)
            && unlock_at.inner() > due_at.inner()
        {
            return Err(AssignmentOverrideDatesError::UnlockAfterDue);
        }
        if let (Some(due_at), Some(lock_at)) = (due_at, lock_at)
            && due_at.inner() > lock_at.inner()
        {
            return Err(AssignmentOverrideDatesError::DueAfterLock);
        }
        if let (Some(unlock_at), Some(lock_at)) = (unlock_at, lock_at)
            && unlock_at.inner() > lock_at.inner()
        {
            return Err(AssignmentOverrideDatesError::UnlockAfterLock);
        }
        Ok(Self {
            unlock_at,
            due_at,
            lock_at,
        })
    }

    pub fn unlock_at(self) -> Option<DueDate> {
        self.unlock_at
    }

    pub fn due_at(self) -> Option<DueDate> {
        self.due_at
    }

    pub fn lock_at(self) -> Option<DueDate> {
        self.lock_at
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum AssignmentOverrideDatesError {
    #[error("override unlock date must be on or before the due date")]
    UnlockAfterDue,
    #[error("override due date must be on or before the lock date")]
    DueAfterLock,
    #[error("override unlock date must be on or before the lock date")]
    UnlockAfterLock,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverrideStudentIds(Vec<UserId>);

impl OverrideStudentIds {
    pub fn new(ids: Vec<UserId>) -> Result<Self, OverrideStudentIdsError> {
        if ids.is_empty() {
            return Err(OverrideStudentIdsError::Empty);
        }
        Ok(Self(ids))
    }

    pub fn iter(&self) -> impl Iterator<Item = &UserId> {
        self.0.iter()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum OverrideStudentIdsError {
    #[error("override must include at least one student id")]
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssignmentOverrideTarget {
    Section(SectionId),
    Students(OverrideStudentIds),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignmentOverride {
    target: AssignmentOverrideTarget,
    dates: AssignmentOverrideDates,
}

impl AssignmentOverride {
    pub fn new(
        target: AssignmentOverrideTarget,
        dates: AssignmentOverrideDates,
    ) -> Self {
        Self { target, dates }
    }

    pub fn target(&self) -> &AssignmentOverrideTarget {
        &self.target
    }

    pub fn dates(&self) -> AssignmentOverrideDates {
        self.dates
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignmentOverrides(Vec<AssignmentOverride>);

impl AssignmentOverrides {
    pub fn new(overrides: Vec<AssignmentOverride>) -> Result<Self, AssignmentOverridesError> {
        if overrides.is_empty() {
            return Err(AssignmentOverridesError::Empty);
        }
        Ok(Self(overrides))
    }

    pub fn iter(&self) -> impl Iterator<Item = &AssignmentOverride> {
        self.0.iter()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum AssignmentOverridesError {
    #[error("assignment overrides must be a non-empty list")]
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GradingPostingPolicy {
    Automatic,
    Manual,
}

impl GradingPostingPolicy {
    pub fn as_str(&self) -> &'static str {
        match self {
            GradingPostingPolicy::Automatic => "automatic",
            GradingPostingPolicy::Manual => "manual",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum GradingPostingPolicyParseError {
    #[error("grading posting policy must be 'automatic' or 'manual'")]
    Invalid,
}

impl FromStr for GradingPostingPolicy {
    type Err = GradingPostingPolicyParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "automatic" | "auto" => Ok(GradingPostingPolicy::Automatic),
            "manual" => Ok(GradingPostingPolicy::Manual),
            _ => Err(GradingPostingPolicyParseError::Invalid),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutedState {
    Muted,
    Unmuted,
}

impl MutedState {
    pub fn as_str(&self) -> &'static str {
        match self {
            MutedState::Muted => "muted",
            MutedState::Unmuted => "unmuted",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum MutedStateParseError {
    #[error("muted state must be 'muted' or 'unmuted'")]
    Invalid,
}

impl FromStr for MutedState {
    type Err = MutedStateParseError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "muted" => Ok(MutedState::Muted),
            "unmuted" => Ok(MutedState::Unmuted),
            _ => Err(MutedStateParseError::Invalid),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AssignmentOverrideDates, GroupAssignmentMode, GroupAssignmentSettings,
        PeerReviewMode, PeerReviewSettings,
    };
    use crate::{DueDate, GroupCategoryId};

    #[test]
    fn peer_review_settings_require_assign_at_for_automatic() {
        let due_at: DueDate = "2025-01-10T00:00:00Z".parse().expect("due");
        let settings = PeerReviewSettings::new(PeerReviewMode::Automatic, None, Some(due_at));
        assert!(settings.is_err());
    }

    #[test]
    fn peer_review_settings_reject_assign_after_due() {
        let assign_at: DueDate = "2025-01-11T00:00:00Z".parse().expect("assign");
        let due_at: DueDate = "2025-01-10T00:00:00Z".parse().expect("due");
        let settings =
            PeerReviewSettings::new(PeerReviewMode::Automatic, Some(assign_at), Some(due_at));
        assert!(settings.is_err());
    }

    #[test]
    fn group_assignment_requires_category() {
        let settings = GroupAssignmentSettings::new(GroupAssignmentMode::Group, None);
        assert!(settings.is_err());
    }

    #[test]
    fn group_assignment_allows_category() {
        let category: GroupCategoryId = "4".parse().expect("category");
        let settings =
            GroupAssignmentSettings::new(GroupAssignmentMode::Group, Some(category));
        assert!(settings.is_ok());
    }

    #[test]
    fn override_dates_validate_order() {
        let unlock_at: DueDate = "2025-01-11T00:00:00Z".parse().expect("unlock");
        let due_at: DueDate = "2025-01-10T00:00:00Z".parse().expect("due");
        let dates = AssignmentOverrideDates::new(Some(unlock_at), Some(due_at), None);
        assert!(dates.is_err());
    }
}
