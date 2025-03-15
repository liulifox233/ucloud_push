use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HomeWork {
    pub id: String,
    pub info: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UndoneList {
    pub site_num: i32,
    pub undone_num: i32,
    pub undone_list: Vec<UndoneListItem>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UndoneListItem {
    pub site_id: i32,
    pub site_name: String,
    pub activity_name: String,
    pub activity_id: String,
    pub r#type: i32,
    pub end_time: String,
    pub assignment_type: i32,
    pub evaluation_status: i32,
    pub is_open_evaluation: i32,
    pub course_info: Option<CourseInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CourseInfo {
    pub id: String,
    pub name: String,
    pub teachers: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub title: String,
    pub start_date: Option<String>,
    pub due_date: Option<String>,
    pub content: Option<String>,
}
