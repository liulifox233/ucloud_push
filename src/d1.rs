use crate::model::{UndoneList, UndoneListItem};
use serde::Deserialize;
use std::collections::HashSet;
use worker::{D1Database, Error};

const CHUNK_SIZE: usize = 100; // 根据 D1 参数限制调整

#[derive(Debug, Deserialize)]
struct ActivityRow {
    activity_id: String,
}

pub async fn filter_pushed_undone_list(
    undone_list: &UndoneList,
    db: &D1Database,
) -> worker::Result<UndoneList> {
    let incoming_ids: Vec<&str> = undone_list
        .undone_list
        .iter()
        .map(|item| item.activity_id.as_str())
        .collect();

    if incoming_ids.is_empty() {
        return Ok(UndoneList {
            site_num: undone_list.site_num,
            undone_num: 0,
            undone_list: Vec::new(),
        });
    }

    // 分块处理查询
    let mut stmts = Vec::new();
    for chunk in incoming_ids.chunks(CHUNK_SIZE) {
        let json_ids = serde_json::to_string(chunk).map_err(|e| Error::RustError(e.to_string()))?;

        let stmt = db
            .prepare(
                "SELECT activity_id FROM activities 
             WHERE activity_id IN (SELECT value FROM json_each(?1))",
            )
            .bind(&[json_ids.into()])?;

        stmts.push(stmt);
    }

    // 批量执行所有查询
    let mut existing_ids = HashSet::new();
    for result_chunk in db.batch(stmts).await? {
        let rows = result_chunk.results::<ActivityRow>()?;
        for row in rows {
            existing_ids.insert(row.activity_id);
        }
    }

    // 过滤未推送的条目
    let filtered: Vec<UndoneListItem> = undone_list
        .undone_list
        .iter()
        .filter(|item| !existing_ids.contains(&item.activity_id))
        .cloned()
        .collect();

    Ok(UndoneList {
        site_num: undone_list.site_num,
        undone_num: filtered.len() as i32,
        undone_list: filtered,
    })
}

pub async fn save_activities_batch(
    items: &[UndoneListItem],
    db: &D1Database,
) -> worker::Result<()> {
    if items.is_empty() {
        return Ok(());
    }

    let mut stmts = Vec::new();

    for chunk in items.chunks(CHUNK_SIZE) {
        let mut placeholders = Vec::new();
        let mut params = Vec::new();

        for (i, item) in chunk.iter().enumerate() {
            placeholders.push(format!(
                "(?{}, ?{}, ?{}, ?{}, ?{}, ?{}, ?{}, ?{}, ?{}, ?{})",
                i * 10 + 1,
                i * 10 + 2,
                i * 10 + 3,
                i * 10 + 4,
                i * 10 + 5,
                i * 10 + 6,
                i * 10 + 7,
                i * 10 + 8,
                i * 10 + 9,
                i * 10 + 10
            ));

            let course_info = item
                .course_info
                .as_ref()
                .and_then(|ci| serde_json::to_string(ci).ok())
                .unwrap_or_default();

            params.extend_from_slice(&[
                item.activity_id.clone().into(),
                item.activity_name.clone().into(),
                item.r#type.into(),
                item.end_time.clone().into(),
                item.assignment_type.into(),
                item.evaluation_status.into(),
                item.is_open_evaluation.into(),
                course_info.into(),
                item.description.clone().unwrap_or_default().into(),
                item.start_time.clone().unwrap_or_default().into(),
            ]);
        }

        let sql = format!(
            "INSERT OR IGNORE INTO activities (
                activity_id, activity_name, type, end_time,
                assignment_type, evaluation_status, 
                is_open_evaluation, course_info, description, start_time
            ) VALUES {}
            ON CONFLICT(activity_id) DO UPDATE SET
                end_time = excluded.end_time,
                evaluation_status = excluded.evaluation_status",
            placeholders.join(",")
        );

        let stmt = db.prepare(&sql).bind(&params)?;
        stmts.push(stmt);
    }

    db.batch(stmts).await?;
    Ok(())
}

pub async fn cleanup_activities(db: &D1Database) -> worker::Result<()> {
    db.exec("DELETE FROM activities").await?;
    Ok(())
}

pub async fn save_state(state: &str, db: &D1Database) -> worker::Result<()> {
    let stmts = vec![db
        .prepare("INSERT OR REPLACE INTO state state VALUES ?1")
        .bind(&[state.into()])?];
    db.batch(stmts).await?;
    Ok(())
}

pub async fn get_state(db: &D1Database) -> worker::Result<Option<String>> {
    let rows = db
        .batch(vec![db.prepare("SELECT state FROM state")])
        .await?[0]
        .results::<(String,)>()?;
    if let Some((state,)) = rows.into_iter().next() {
        Ok(Some(state))
    } else {
        Ok(None)
    }
}
