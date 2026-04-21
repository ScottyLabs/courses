use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Courses::Table)
                    .if_not_exists()
                    .col(pk_auto(Courses::CourseId))
                    .col(char_len(Courses::CourseNumber, 10))
                    .col(integer(Courses::DepId))
                    .col(text(Courses::Description))
                    .col(string_len(Courses::Name, 255))
                    .col(double(Courses::Units))
                    .col(json_binary(Courses::Prerequisites))
                    .col(json_binary(Courses::Corequisites))
                    .col(json_binary(Courses::Unlocks))
                    .col(json_binary(Courses::CustomFields))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Courses::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum Courses {
    Table,
    CourseId,
    CourseNumber,
    DepId,
    Description,
    Name,
    Units,
    Prerequisites,
    Corequisites,
    Unlocks,
    CustomFields,
}
