use baizekit::app::anyhow;
use baizekit_app::application::{ComponentKey, InitStrategy};
use std::any::TypeId;
use tracing::info;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use baizekit::app::new_app;
    use baizekit::component::{DbComponent, LogComponent};

    new_app!()
        .register_component_factory(None::<&str>, LogComponent::new)
        .register_component_factory(None::<&str>, DbComponent::new)
        .set_default_handler(|_app, _| {
            let fut = async {
                info!("Default handler executed.");
                Ok(())
            };
            (
                InitStrategy::Deny(vec![ComponentKey {
                    type_id: TypeId::of::<DbComponent>(),
                    label: "default".to_string(),
                }]),
                fut,
            )
        })
        .run()
        .await
}
