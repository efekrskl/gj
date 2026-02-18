use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct EmptyObject {}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectType {
    Database,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ParentType {
    PageId,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RichTextType {
    Text,
}

#[derive(Debug, Serialize)]
pub struct RichText<'a> {
    #[serde(rename = "type")]
    pub kind: RichTextType,
    pub text: TextContent<'a>,
}

#[derive(Debug, Serialize)]
pub struct TextContent<'a> {
    pub content: &'a str,
}

#[derive(Debug, Serialize)]
pub struct CreatePageRequest<'a> {
    pub parent: CreatePageParent<'a>,
    pub properties: CreatePageProperties<'a>,
}

#[derive(Debug, Serialize)]
pub struct CreatePageParent<'a> {
    pub database_id: &'a str,
}

#[derive(Debug, Serialize)]
pub struct CreatePageProperties<'a> {
    #[serde(rename = "Name")]
    pub name: TitleProperty<'a>,
    #[serde(rename = "Date")]
    pub date: DateProperty<'a>,
}

#[derive(Debug, Serialize)]
pub struct TitleProperty<'a> {
    pub title: Vec<RichText<'a>>,
}

#[derive(Debug, Serialize)]
pub struct DateProperty<'a> {
    pub date: DateValue<'a>,
}

#[derive(Debug, Serialize)]
pub struct DateValue<'a> {
    pub start: &'a str, // RFC3339
}

#[derive(Debug, Serialize)]
pub struct QueryDatabaseRequest<'a> {
    pub filter: TitleEqualsFilter<'a>,
}

#[derive(Debug, Serialize)]
pub struct TitleEqualsFilter<'a> {
    pub property: &'a str, // "Name"
    pub title: EqualsFilter<'a>,
}

#[derive(Debug, Serialize)]
pub struct EqualsFilter<'a> {
    pub equals: &'a str,
}

#[derive(Debug, Serialize)]
pub struct AppendChildrenRequest<'a> {
    pub children: Vec<Block<'a>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BlockType {
    BulletedListItem,
}

#[derive(Debug, Serialize)]
pub struct Block<'a> {
    #[serde(rename = "type")]
    pub kind: BlockType,
    pub bulleted_list_item: BulletedListItem<'a>,
}

#[derive(Debug, Serialize)]
pub struct BulletedListItem<'a> {
    pub rich_text: Vec<RichText<'a>>,
}

#[derive(Debug, Serialize)]
pub struct CreateDatabaseRequest<'a> {
    pub is_inline: bool,
    pub parent: CreateDatabaseParent<'a>,
    pub title: Vec<RichText<'a>>,
    pub properties: DatabaseProperties<'a>,
}

#[derive(Debug, Serialize)]
pub struct CreateDatabaseParent<'a> {
    #[serde(rename = "type")]
    pub kind: ParentType, // PageId
    pub page_id: &'a str,
}

#[derive(Debug, Serialize)]
pub struct DatabaseProperties<'a> {
    #[serde(rename = "Name")]
    pub name: DatabaseTitleProperty,
    #[serde(rename = "Date")]
    pub date: DatabaseDateProperty,
    #[serde(rename = "Highlights")]
    pub highlights: DatabaseRichTextProperty,
    #[serde(rename = "Tag")]
    pub tag: DatabaseMultiSelectProperty<'a>,
}

#[derive(Debug, Serialize)]
pub struct DatabaseTitleProperty {
    pub title: EmptyObject, // {}
}

#[derive(Debug, Serialize)]
pub struct DatabaseDateProperty {
    pub date: EmptyObject, // {}
}

#[derive(Debug, Serialize)]
pub struct DatabaseRichTextProperty {
    pub rich_text: EmptyObject, // {}
}

#[derive(Debug, Serialize)]
pub struct DatabaseMultiSelectProperty<'a> {
    pub multi_select: MultiSelectOptions<'a>,
}

#[derive(Debug, Serialize)]
pub struct MultiSelectOptions<'a> {
    pub options: Vec<SelectOption<'a>>,
}

#[derive(Debug, Serialize)]
pub struct SelectOption<'a> {
    pub name: &'a str,
    pub color: &'a str,
}

#[derive(Debug, Serialize)]
pub struct SearchRequest<'a> {
    pub filter: SearchFilter<'a>,
}

#[derive(Debug, Serialize)]
pub struct SearchFilter<'a> {
    pub value: ObjectType, // Database
    pub property: &'a str, // "object"
}

#[derive(Debug, Serialize)]
pub struct UpdatePageTagsRequest<'a> {
    pub properties: UpdatePageTagsProperties<'a>,
}

#[derive(Debug, Serialize)]
pub struct UpdatePageTagsProperties<'a> {
    #[serde(rename = "Tag")]
    pub tag: UpdateMultiSelect<'a>,
}

#[derive(Debug, Serialize)]
pub struct UpdateMultiSelect<'a> {
    pub multi_select: Vec<TagName<'a>>,
}

#[derive(Debug, Serialize)]
pub struct TagName<'a> {
    pub name: &'a str,
}

#[derive(Debug, Deserialize)]
pub struct IdResponse {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct QueryResponse<T> {
    pub results: Vec<T>,
}

#[derive(Debug, Deserialize)]
pub struct PageStub {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchDatabaseResult>,
}

#[derive(Debug, Deserialize)]
pub struct SearchDatabaseResult {
    pub id: String,
    pub title: Vec<SearchTitlePart>,
}

#[derive(Debug, Deserialize)]
pub struct SearchTitlePart {
    pub plain_text: String,
}

#[derive(Debug, Deserialize)]
pub struct PageResponse {
    pub properties: PageProperties,
}

#[derive(Debug, Deserialize)]
pub struct PageProperties {
    #[serde(rename = "Tag")]
    pub tag: PageMultiSelect,
}

#[derive(Debug, Deserialize)]
pub struct PageMultiSelect {
    pub multi_select: Vec<PageSelectOption>,
}

#[derive(Debug, Deserialize)]
pub struct PageSelectOption {
    pub name: String,
}
