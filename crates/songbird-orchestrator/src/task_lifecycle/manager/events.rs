// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright (c) 2024-2026 ecoPrimals

//! Task lifecycle broadcast events.

use super::super::{TaskId, TowerId, UserId};
use std::sync::Arc;

/// Task event for streaming
#[derive(Debug, Clone)]
pub enum TaskEvent {
    Created {
        task_id: TaskId,
        owner: UserId,
    },
    Started {
        task_id: TaskId,
        tower: TowerId,
    },
    ProgressUpdated {
        task_id: TaskId,
        progress: f32,
    },
    Paused {
        task_id: TaskId,
    },
    Resumed {
        task_id: TaskId,
        tower: TowerId,
    },
    CheckpointCreated {
        task_id: TaskId,
        checkpoint_id: Arc<str>,
    },
    Completed {
        task_id: TaskId,
    },
    Failed {
        task_id: TaskId,
        error: Arc<str>,
    },
    Cancelled {
        task_id: TaskId,
        reason: Option<Arc<str>>,
    },
}
