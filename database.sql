CREATE TABLE IF NOT EXISTS activities (
    activity_id TEXT PRIMARY KEY,
    pushed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    activity_name TEXT NOT NULL,
    type INTEGER NOT NULL,
    start_time TEXT NOT NULL,
    end_time TEXT NOT NULL,
    assignment_type INTEGER NOT NULL,
    evaluation_status INTEGER NOT NULL,
    is_open_evaluation INTEGER NOT NULL,
    course_info TEXT,
    description TEXT,
);