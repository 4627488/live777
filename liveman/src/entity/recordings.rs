use sea_orm::{JsonValue, entity::prelude::*};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "recordings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub stream: String,
    pub record: String,
    pub mpd_path: String,
    pub start_time: Option<i64>,
    pub duration: Option<f64>,
    pub meta: Option<JsonValue>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
