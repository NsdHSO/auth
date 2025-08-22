use base64::engine::general_purpose;
use base64::Engine;
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use jsonwebtoken::{self, Algorithm, EncodingKey, Header};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

use std::collections::HashSet;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, JoinType, QueryFilter, QuerySelect, RelationTrait};

use crate::entity::{
    permissions, role_permissions, roles, user_permission_overrides, user_roles, users,
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenDetails {
    pub user_id: Uuid,
    pub token_uuid: Uuid,
    pub expires_in: Option<i64>,
    pub token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenClaims {
    pub sub: String,
    pub token_uuid: String,
    pub perms: Vec<String>,
    pub roles: Vec<String>,
    pub exp: i64,
    pub iat: i64,
    pub nbf: i64,
}

pub async fn compute_roles_and_permissions(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Result<(Vec<String>, Vec<String>), sea_orm::DbErr> {
    let mut roles_set: HashSet<String> = HashSet::new();
    let mut perms_set: HashSet<String> = HashSet::new();

    // Base role from users.role
    if let Some(user) = users::Entity::find_by_id(user_id).one(db).await? {
        roles_set.insert(user.role.as_str().to_string());
    }

    // Roles via user_roles
    let assigned_roles = roles::Entity::find()
        .join(JoinType::InnerJoin, roles::Relation::UserRoles.def())
        .filter(user_roles::Column::UserId.eq(user_id))
        .all(db)
        .await?;
    for r in assigned_roles {
        roles_set.insert(r.code);
    }

    // Permissions via user_roles -> role_permissions -> permissions
    let perms_models_via_user_roles = permissions::Entity::find()
        .join(JoinType::InnerJoin, permissions::Relation::RolePermissions.def())
        .join(JoinType::InnerJoin, role_permissions::Relation::Roles.def())
        .join(JoinType::InnerJoin, roles::Relation::UserRoles.def())
        .filter(user_roles::Column::UserId.eq(user_id))
        .all(db)
        .await?;
    for p in perms_models_via_user_roles {
        perms_set.insert(p.code);
    }

    // Permissions via base role (if mapped in roles table)
    for base_role in roles_set.clone().into_iter() {
        let base_perm_models = permissions::Entity::find()
            .join(JoinType::InnerJoin, permissions::Relation::RolePermissions.def())
            .join(JoinType::InnerJoin, role_permissions::Relation::Roles.def())
            .filter(roles::Column::Code.eq(base_role))
            .all(db)
            .await?;
        for p in base_perm_models {
            perms_set.insert(p.code);
        }
    }

    // Overrides: allow
    let allow_models = permissions::Entity::find()
        .join(
            JoinType::InnerJoin,
            permissions::Relation::UserPermissionOverrides.def(),
        )
        .filter(user_permission_overrides::Column::UserId.eq(user_id))
        .filter(user_permission_overrides::Column::Allow.eq(true))
        .all(db)
        .await?;
    for p in allow_models {
        perms_set.insert(p.code);
    }

    // Overrides: deny
    let deny_models = permissions::Entity::find()
        .join(
            JoinType::InnerJoin,
            permissions::Relation::UserPermissionOverrides.def(),
        )
        .filter(user_permission_overrides::Column::UserId.eq(user_id))
        .filter(user_permission_overrides::Column::Allow.eq(false))
        .all(db)
        .await?;
    for p in deny_models {
        perms_set.remove(&p.code);
    }

    let mut roles: Vec<String> = roles_set.into_iter().collect();
    roles.sort();
    let mut perms: Vec<String> = perms_set.into_iter().collect();
    perms.sort();

    Ok((roles, perms))
}

pub fn generate_jwt_token(
    user_id: Uuid,
    ttl: i64,
    private_key: String,
    perms: Vec<String>,
    roles: Vec<String>,
) -> Result<TokenDetails, jsonwebtoken::errors::Error> {
    let bytes_private_key = general_purpose::STANDARD.decode(private_key).unwrap();
    let decoded_private_key = String::from_utf8(bytes_private_key).unwrap();

    let now = chrono::Utc::now();
    let mut token_details = TokenDetails {
        user_id,
        token_uuid: Uuid::new_v4(),
        expires_in: Some((now + chrono::Duration::minutes(ttl)).timestamp()),
        token: None,
    };

    let claims = TokenClaims {
        sub: token_details.user_id.to_string(),
        token_uuid: token_details.token_uuid.to_string(),
        perms,
        roles,
        exp: token_details.expires_in.unwrap(),
        iat: now.timestamp(),
        nbf: now.timestamp(),
    };

    let header = Header::new(Algorithm::RS256);
    let token = jsonwebtoken::encode(
        &header,
        &claims,
        &EncodingKey::from_rsa_pem(decoded_private_key.as_bytes())?,
    )?;
    token_details.token = Some(token);
    Ok(token_details)
}

pub fn verify_jwt_token(
    public_key: String,
    token: &str,
) -> Result<TokenDetails, jsonwebtoken::errors::Error> {
    let bytes_public_key = general_purpose::STANDARD.decode(public_key)?;
    let decoded_public_key = String::from_utf8(bytes_public_key)?;

    let validation = jsonwebtoken::Validation::new(Algorithm::RS256);

    let decoded = jsonwebtoken::decode::<TokenClaims>(
        token,
        &jsonwebtoken::DecodingKey::from_rsa_pem(decoded_public_key.as_bytes())?,
        &validation,
    )?;

    let user_id = Uuid::parse_str(decoded.claims.sub.as_str()).unwrap();
    let token_uuid = Uuid::parse_str(decoded.claims.token_uuid.as_str()).unwrap();

    Ok(TokenDetails {
        token: None,
        token_uuid,
        user_id,
        expires_in: None,
    })
}
// helper: generate opaque refresh (raw + hash)
pub fn hash_refresh(raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    STANDARD.encode(hasher.finalize())
}
pub fn generate_opaque_refresh() -> (String, String) {
    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    let raw = URL_SAFE_NO_PAD.encode(&bytes);
    let hash = hash_refresh(&raw);
    (raw, hash)
}
