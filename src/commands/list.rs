use crate::error::DevCloneError;
use crate::info;
use crate::registry::Registry;
use comfy_table::{Table, presets::NOTHING};
use std::time::{SystemTime, UNIX_EPOCH};

fn format_age(created_at: u64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let seconds = now.saturating_sub(created_at);

    match seconds {
        0..=59 => format!("{seconds}s"),
        60..=3599 => format!("{}m", seconds / 60),
        3600..=86399 => format!("{}h", seconds / 3600),
        _ => format!("{}d", seconds / 86400),
    }
}

pub fn list_instances() -> Result<(), DevCloneError> {
    let registry = Registry::load()?;
    let instances = &registry.instances;

    if instances.is_empty() {
        info!("No managed instances found.");
    } else {
        let mut table = Table::new();

        table
            .load_style(NOTHING)
            .set_header(["NAME", "MODE", "REVISION", "STATUS", "AGE", "SOURCE"]);

        for instance in instances {
            table.add_row([
                &instance.name,
                &instance.mode.to_string(),
                &instance.revision,
                &instance.status.to_string(),
                &format_age(instance.created_at),
                &instance.source.display().to_string(),
            ]);
        }
        println!("{}", table);
    }

    Ok(())
}
