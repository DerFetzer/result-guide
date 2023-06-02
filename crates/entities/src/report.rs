//! SeaORM Entity. Generated by sea-orm-codegen 0.9.1

use std::fmt::Display;

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "report")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(default)]
    pub id: i32,
    pub date: DateTimeWithTimeZone,
    pub project: String,
    pub name: String,
    pub verdict: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::test_step::Entity")]
    TestStep,
}

impl Related<super::test_step::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TestStep.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Display for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {} ({})", self.name, self.verdict, self.date)
    }
}
