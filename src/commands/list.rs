use crate::error::DevCloneError;
use crate::info;
use crate::registry::Registry;

pub fn list_instances() -> Result<(), DevCloneError> {
    let registry = Registry::load()?;
    let instances = &registry.instances;

    if instances.is_empty() {
        info!("No managed instances found.");
    } else {
        println!("NAME | MODE | REVISION | STATUS | AGE | SOURCE");
        for instance in instances {
            println!("{instance}");
        }
    }

    Ok(())
}
