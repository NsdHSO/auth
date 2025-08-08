use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "user_status")]
pub enum UserStatus {
    #[sea_orm(string_value = "ACTIVE")]
    Active,

    #[sea_orm(string_value = "INACTIVE")]
    Inactive,

    #[sea_orm(string_value = "SUSPENDED")]
    Suspended,

    #[sea_orm(string_value = "PENDING_VERIFICATION")]
    PendingVerification,
}

#[allow(dead_code)]
impl UserStatus {
    pub fn is_active(&self) -> bool {
        matches!(self, UserStatus::Active)
    }

    pub fn is_suspended(&self) -> bool {
        matches!(self, UserStatus::Suspended)
    }
}
