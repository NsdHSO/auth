use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "role_permissions", schema_name = "auth")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub role_id: Uuid,

    #[sea_orm(primary_key, auto_increment = false)]
    pub permission_id: Uuid,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::roles::Entity",
        from = "Column::RoleId",
        to = "super::roles::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Roles,

    #[sea_orm(
        belongs_to = "super::permissions::Entity",
        from = "Column::PermissionId",
        to = "super::permissions::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Permissions,
}

impl Related<super::roles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Roles.def()
    }
}

impl Related<super::permissions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Permissions.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

