use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Homework {
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

    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_overtime_commit: Option<bool>,
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
    pub project_id: String,
    pub start_date: Option<String>,
    pub due_date: Option<String>,
    pub content: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    pub resource_id: String,
    pub resource_name: String,
    pub resource_type: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Detail {
    pub id: String,
    pub assignment_title: String,
    pub assignment_content: String,
    pub assignment_comment: String,
    pub class_name: String,
    pub chapter_name: String,
    pub assignment_type: i32,
    pub no_submit_num: i32,
    pub total_num: i32,
    pub stay_read_num: i32,
    pub already_read_num: i32,
    pub is_group_excellent: i32,
    pub assignment_begin_time: String,
    pub assignment_end_time: String,
    pub is_overtime_commit: i32,
    pub assignment_status: i32,
    pub team_id: i32,
    pub is_open_evaluation: i32,
    pub status: i32,
    pub group_score: f64,
    pub assignment_score: f64,
    pub assignment_resource: Vec<Resource>,
    pub assignment_mutual_evaluation: serde_json::Value,
    pub course_info: Option<serde_json::Value>,
    pub key: Option<String>,
    pub resource: Option<Vec<ResourceDetail>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ResourceDetail {
    pub storage_id: String,
    pub name: String,
    pub ext: String,
    pub id: String,
}
