use std::process::Command;
use task_hookrs::import::import;
use task_hookrs::task::Task;

use crate::error::{Result, TSError};

pub struct TaskClient {
    pub tasks: Vec<Task>,
    pub filter: String,
}

impl TaskClient {
    pub fn new(filter: &str) -> Result<Self> {
        let mut client = TaskClient {
            tasks: Vec::new(),
            filter: filter.to_string(),
        };

        client.refresh_tasks()?;
        Ok(client)
    }

    pub fn refresh_tasks(&mut self) -> Result<()> {
        let mut task = Command::new("task");

        task.arg("rc.json.array=on");
        task.arg("rc.confirmation=off");
        task.arg("export");

        let filter = format!(
            "{} {}",
            self.filter,
            self.get_context()?.unwrap_or("".to_string())
        );

        match shlex::split(&filter) {
            Some(cmd) => {
                for s in cmd {
                    task.arg(&s);
                }
            }
            None => {
                task.arg("");
            }
        }

        let output = task.output()?;

        match import(String::from_utf8_lossy(&output.stdout).as_bytes()) {
            Ok(tasks) => {
                self.tasks = tasks;
                Ok(())
            }
            Err(_) => Err(TSError::Error(
                "could not load tasks from output".to_string(),
            )),
        }
    }

    fn get_context(&self) -> Result<Option<String>> {
        let output = Command::new("task")
            .arg("_get")
            .arg("rc.context")
            .output()?;
        let context = String::from_utf8_lossy(&output.stdout)
            .to_string()
            .strip_suffix('\n')
            .unwrap_or("")
            .to_string();

        if context.len() == 0 {
            return Ok(None);
        }

        let output = Command::new("task")
            .arg("_get")
            .arg(format!("rc.context.{}", context))
            .output()?;
        match String::from_utf8_lossy(&output.stdout)
            .to_string()
            .strip_suffix('\n')
        {
            Some(filter) => Ok(Some(filter.to_string())),
            // FIXME: make error - MCL - 2020-11-17
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
